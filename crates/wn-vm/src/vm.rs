//! Stack VM para bytecode de WN++.
//!
//! Ejecuta un [`Chunk`] instrucción por instrucción manteniendo un stack de valores,
//! un call stack de [`CallFrame`]s, y un mapa de globales.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{chunk::Chunk, opcode::OpCode, value::Value};

#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("La wea '{0}' no existe, papito.")]
    VarNoDefinida(String),

    #[error("No podi hacer '{op}' entre un '{tipo_izq}' y un '{tipo_der}', pedazo de saco wea.")]
    TiposIncompatibles {
        op: &'static str,
        tipo_izq: &'static str,
        tipo_der: &'static str,
    },

    #[error("No podi negar un '{0}'.")]
    NegacionInvalida(&'static str),

    #[error("División por cero, wn.")]
    DivisionPorCero,

    #[error("'{0}' no es llamable.")]
    NoLlamable(&'static str),

    #[error("Índice {indice} fuera de rango (largo: {largo}).")]
    IndiceInvalido { indice: i64, largo: usize },

    #[error("Los índices deben ser números, no '{0}'.")]
    IndiceTipoInvalido(&'static str),

    #[error(
        "Opcode inválido: {0:#04x} - bug del compilador, reportar issue en https://github.com/cuervolu/wn/issues"
    )]
    OpcodeInvalido(u8),

    #[error(
        "Stack underflow - bug del compilador, reportar issue en https://github.com/cuervolu/wn/issues"
    )]
    StackUnderflow,
}

/// Un frame de ejecución activo.
///
/// Cada llamada a función crea un frame nuevo. El script raíz también
/// corre en un frame con `base_slot = 0`.
///
/// Cuando `Llamar`/`Devolver` estén implementados, el VM:
///   - En `Llamar`: guarda `ip` del frame actual, push nuevo `CallFrame`.
///   - En `Devolver`: pop el frame actual, restaura `ip` del frame anterior.
#[derive(Debug)]
pub struct CallFrame {
    /// Instruction pointer: byte offset en el chunk de este frame.
    /// Se sincroniza de/hacia la variable local `ip` del loop de ejecución.
    pub ip: usize,
    /// Dónde empiezan los locales de este frame en el value stack.
    /// `ObtenerLocal(i)` accede a `stack[base_slot + i]`.
    pub base_slot: usize,
}

pub struct VM {
    stack: Vec<Value>,
    /// Variables globales. Persisten entre llamadas a `run` (necesario para el REPL).
    pub globals: HashMap<String, Value>,
    pub frames: Vec<CallFrame>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            frames: Vec::with_capacity(64),
        }
    }

    /// Ejecuta un chunk en un frame nuevo.
    /// Los globales persisten entre llamadas sucesivas.
    pub fn run(&mut self, chunk: &Chunk) -> Result<Value, VmError> {
        self.frames.push(CallFrame {
            ip: 0,
            base_slot: 0,
        });

        // `ip` vive local al loop para acceso sin borrow del Vec de frames.
        // Se sincroniza de vuelta al frame en los puntos de salida (RetornarNada, Devolver).
        // Cuando Llamar esté implementado, también se sincronizará al hacer push/pop de frame.
        let mut ip = 0usize;
        let base_slot = self.frames.last().unwrap().base_slot;

        let result = self.execute(chunk, &mut ip, base_slot);

        if let Some(frame) = self.frames.last_mut() {
            frame.ip = ip;
        }
        self.frames.pop();
        result
    }

    fn execute(
        &mut self,
        chunk: &Chunk,
        ip: &mut usize,
        base_slot: usize,
    ) -> Result<Value, VmError> {
        loop {
            let byte = chunk.code[*ip];
            *ip += 1;

            let op = OpCode::try_from(byte).map_err(VmError::OpcodeInvalido)?;

            match op {
                OpCode::Constante => {
                    let idx = read_u16(&chunk.code, ip) as usize;
                    self.push(chunk.constants[idx].clone());
                }

                OpCode::Nada => self.push(Value::Nada),
                OpCode::Verdad => self.push(Value::Booleano(true)),
                OpCode::Falso => self.push(Value::Booleano(false)),

                OpCode::Pop => {
                    self.pop()?;
                }

                OpCode::DefinirGlobal => {
                    let nombre = self.read_nombre_constante(chunk, ip);
                    let val = self.pop()?;
                    self.globals.insert(nombre, val);
                }

                OpCode::ObtenerGlobal => {
                    let nombre = self.read_nombre_constante(chunk, ip);
                    let val = self
                        .globals
                        .get(&nombre)
                        .ok_or_else(|| VmError::VarNoDefinida(nombre.clone()))?
                        .clone();
                    self.push(val);
                }

                OpCode::AsignarGlobal => {
                    let nombre = self.read_nombre_constante(chunk, ip);
                    if !self.globals.contains_key(&nombre) {
                        return Err(VmError::VarNoDefinida(nombre));
                    }
                    let val = self.peek()?.clone();
                    self.globals.insert(nombre, val);
                }

                OpCode::ObtenerLocal => {
                    let slot = read_byte(&chunk.code, ip) as usize;
                    let val = self.stack[base_slot + slot].clone();
                    self.push(val);
                }

                OpCode::AsignarLocal => {
                    let slot = read_byte(&chunk.code, ip) as usize;
                    let val = self.peek()?.clone();
                    self.stack[base_slot + slot] = val;
                }

                OpCode::Suma => self.op_suma()?,
                OpCode::Resta => self.op_binaria_numerica("-", |a, b| a - b)?,
                OpCode::Mul => self.op_binaria_numerica("*", |a, b| a * b)?,
                OpCode::Div => self.op_div()?,
                OpCode::Mod => self.op_mod()?,

                OpCode::Neg => {
                    let val = self.pop()?;
                    match val {
                        Value::Numero(n) => self.push(Value::Numero(-n)),
                        other => return Err(VmError::NegacionInvalida(other.tipo_nombre())),
                    }
                }

                OpCode::No => {
                    let val = self.pop()?;
                    self.push(Value::Booleano(!val.es_verdadero()));
                }

                OpCode::Eq => {
                    let der = self.pop()?;
                    let izq = self.pop()?;
                    self.push(Value::Booleano(izq == der));
                }

                OpCode::Neq => {
                    let der = self.pop()?;
                    let izq = self.pop()?;
                    self.push(Value::Booleano(izq != der));
                }

                OpCode::Lt => self.op_comparacion("<", |a, b| a < b)?,
                OpCode::Gt => self.op_comparacion(">", |a, b| a > b)?,
                OpCode::Lte => self.op_comparacion("<=", |a, b| a <= b)?,
                OpCode::Gte => self.op_comparacion(">=", |a, b| a >= b)?,

                OpCode::Saltar => {
                    let offset = read_u16(&chunk.code, ip) as usize;
                    *ip += offset;
                }

                OpCode::SaltarSiFalso => {
                    let offset = read_u16(&chunk.code, ip) as usize;
                    if !self.peek()?.es_verdadero() {
                        *ip += offset;
                    }
                }

                OpCode::Loop => {
                    let offset = read_u16(&chunk.code, ip) as usize;
                    *ip -= offset;
                }

                OpCode::Lorea => {
                    let val = self.pop()?;
                    let stdout = io::stdout();
                    writeln!(stdout.lock(), "{val}").ok();
                }

                OpCode::RetornarNada => return Ok(Value::Nada),

                OpCode::Devolver => {
                    // A nivel de script termina la ejecución.
                    // Con call frames: pop frame y continuar en el caller.
                    return self.pop();
                }

                OpCode::ConstruirLista
                | OpCode::ConstruirMapa
                | OpCode::ObtenerIndice
                | OpCode::AsignarIndice => todo!("colecciones — pendiente"),

                OpCode::Llamar => todo!("llamadas a funciones — próximo milestone"),
            }
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackUnderflow)
    }

    fn read_nombre_constante(&self, chunk: &Chunk, ip: &mut usize) -> String {
        let idx = read_u16(&chunk.code, ip) as usize;
        match &chunk.constants[idx] {
            Value::Texto(s) => s.to_string(),
            _ => panic!("constante de nombre no es texto — bug del compilador"),
        }
    }

    fn op_suma(&mut self) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        let resultado = match (izq, der) {
            (Value::Numero(a), Value::Numero(b)) => Value::Numero(a + b),
            (Value::Texto(a), Value::Texto(b)) => Value::from(format!("{a}{b}").as_str()),
            (Value::Texto(a), Value::Numero(b)) => {
                Value::from(format!("{a}{}", Value::Numero(b)).as_str())
            }
            (Value::Numero(a), Value::Texto(b)) => {
                Value::from(format!("{}{b}", Value::Numero(a)).as_str())
            }
            (izq, der) => {
                return Err(VmError::TiposIncompatibles {
                    op: "+",
                    tipo_izq: izq.tipo_nombre(),
                    tipo_der: der.tipo_nombre(),
                });
            }
        };
        self.push(resultado);
        Ok(())
    }

    fn op_binaria_numerica(
        &mut self,
        op: &'static str,
        f: impl Fn(f64, f64) -> f64,
    ) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        match (izq, der) {
            (Value::Numero(a), Value::Numero(b)) => {
                self.push(Value::Numero(f(a, b)));
                Ok(())
            }
            (izq, der) => Err(VmError::TiposIncompatibles {
                op,
                tipo_izq: izq.tipo_nombre(),
                tipo_der: der.tipo_nombre(),
            }),
        }
    }

    fn op_div(&mut self) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        match (izq, der) {
            (Value::Numero(_), Value::Numero(0.0)) => Err(VmError::DivisionPorCero),
            (Value::Numero(a), Value::Numero(b)) => {
                self.push(Value::Numero(a / b));
                Ok(())
            }
            (izq, der) => Err(VmError::TiposIncompatibles {
                op: "/",
                tipo_izq: izq.tipo_nombre(),
                tipo_der: der.tipo_nombre(),
            }),
        }
    }

    fn op_mod(&mut self) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        match (izq, der) {
            (Value::Numero(_), Value::Numero(0.0)) => Err(VmError::DivisionPorCero),
            (Value::Numero(a), Value::Numero(b)) => {
                self.push(Value::Numero(a % b));
                Ok(())
            }
            (izq, der) => Err(VmError::TiposIncompatibles {
                op: "%",
                tipo_izq: izq.tipo_nombre(),
                tipo_der: der.tipo_nombre(),
            }),
        }
    }

    fn op_comparacion(
        &mut self,
        op: &'static str,
        f: impl Fn(f64, f64) -> bool,
    ) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        match (izq, der) {
            (Value::Numero(a), Value::Numero(b)) => {
                self.push(Value::Booleano(f(a, b)));
                Ok(())
            }
            (izq, der) => Err(VmError::TiposIncompatibles {
                op,
                tipo_izq: izq.tipo_nombre(),
                tipo_der: der.tipo_nombre(),
            }),
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

fn read_byte(code: &[u8], ip: &mut usize) -> u8 {
    let b = code[*ip];
    *ip += 1;
    b
}

fn read_u16(code: &[u8], ip: &mut usize) -> u16 {
    let hi = code[*ip] as u16;
    let lo = code[*ip + 1] as u16;
    *ip += 2;
    (hi << 8) | lo
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::compilar;
    use wn::{lexer::tokenizar, parser::parsear};

    fn run_src(src: &str) -> VM {
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        let chunk = compilar(&stmts).unwrap();
        let mut vm = VM::new();
        vm.run(&chunk).unwrap();
        vm
    }

    #[test]
    fn global_aritmetica() {
        let vm = run_src("wea x = 10 + 20");
        assert_eq!(vm.globals["x"], Value::Numero(30.0));
    }

    #[test]
    fn global_string() {
        let vm = run_src(r#"wea s = "wena " + "wn++""#);
        assert_eq!(vm.globals["s"], Value::from("wena wn++"));
    }

    #[test]
    fn cachai_rama_verdadera() {
        let vm = run_src("wea x = 0\ncachai (verdad) { x = 5 }");
        assert_eq!(vm.globals["x"], Value::Numero(5.0));
    }

    #[test]
    fn cachai_rama_falsa_si_no() {
        let vm = run_src("wea x = 0\ncachai (falso) { x = 1 } si no { x = 2 }");
        assert_eq!(vm.globals["x"], Value::Numero(2.0));
    }

    #[test]
    fn cachai_no_entra_si_falso() {
        let vm = run_src("wea x = 0\ncachai (falso) { x = 99 }");
        assert_eq!(vm.globals["x"], Value::Numero(0.0));
    }

    #[test]
    fn mientras_cuenta() {
        let vm = run_src("wea i = 0\nmientras (i < 5) { i = i + 1 }");
        assert_eq!(vm.globals["i"], Value::Numero(5.0));
    }

    #[test]
    fn short_circuit_y_no_evalua_der() {
        let vm = run_src("wea r = falso y verdad");
        assert_eq!(vm.globals["r"], Value::Booleano(false));
    }

    #[test]
    fn short_circuit_o_no_evalua_der() {
        let vm = run_src("wea r = verdad o falso");
        assert_eq!(vm.globals["r"], Value::Booleano(true));
    }

    #[test]
    fn local_dentro_de_scope() {
        let vm = run_src("wea resultado = 0\ncachai (verdad) { wea x = 42\nresultado = x }");
        assert_eq!(vm.globals["resultado"], Value::Numero(42.0));
        assert!(!vm.globals.contains_key("x"));
    }

    #[test]
    fn call_frame_script_nivel() {
        let vm = run_src("wea x = 1 + 2");
        assert!(vm.frames.is_empty());
        assert_eq!(vm.globals["x"], Value::Numero(3.0));
    }

    #[test]
    fn duro_no_reasignable() {
        let tokens = tokenizar("duro PI = 3\nPI = 4").unwrap();
        let stmts = parsear(tokens, "duro PI = 3\nPI = 4", "<test>").unwrap();
        assert!(compilar(&stmts).is_err());
    }
}
