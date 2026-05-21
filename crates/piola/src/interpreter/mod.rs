pub mod env;
pub mod error;
pub mod value;

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Write},
    rc::Rc,
};

use crate::{
    ast::{Expr, OpBin, OpUn, Stmt},
    error::PiolaError,
};

use env::Entorno;
use error::RuntimeError;
use value::{Nativa, Valor};

fn nuevo_scope(padre: &Rc<RefCell<Entorno>>) -> Rc<RefCell<Entorno>> {
    Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(padre))))
}

fn resolver_indice(i: i64, len: usize) -> Result<usize, RuntimeError> {
    let idx = if i < 0 {
        let pos = len as i64 + i;
        if pos < 0 {
            return Err(RuntimeError::IndiceInvalido(i, len));
        }
        pos as usize
    } else {
        i as usize
    };
    if idx >= len {
        return Err(RuntimeError::IndiceInvalido(i, len));
    }
    Ok(idx)
}

fn numeric_idx(idx: &Valor, contexto: &str) -> Result<i64, RuntimeError> {
    match idx {
        Valor::Numero(n) => Ok(*n as i64),
        other => Err(RuntimeError::TipoInvalido(format!(
            "Los índices de {contexto} deben ser números, no '{}'.",
            other.tipo_nombre()
        ))),
    }
}

fn numeric_op_checked(
    lhs: Valor,
    rhs: Valor,
    op_sym: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<Valor, RuntimeError> {
    if let (Valor::Numero(_), Valor::Numero(b)) = (&lhs, &rhs)
        && *b == 0.0
    {
        return Err(RuntimeError::DivisionPorCero);
    }
    numeric_op(lhs, rhs, op_sym, f)
}

macro_rules! exec_body_loop {
    ($self:ident, $cuerpo:expr, $child:expr) => {
        match $self.eval_stmts($cuerpo, $child) {
            Ok(_) => {}
            Err(RuntimeError::Cortala) => break,
            Err(RuntimeError::Sigue) => continue,
            Err(e) => return Err(e),
        }
    };
}

pub struct Interprete {
    global: Rc<RefCell<Entorno>>,
    salida: Rc<RefCell<Box<dyn Write>>>,
}

impl Interprete {
    pub fn nuevo() -> Self {
        Self::con_salida(io::stdout())
    }

    pub fn con_salida<W>(salida: W) -> Self
    where
        W: Write + 'static,
    {
        let global = Rc::new(RefCell::new(Entorno::nuevo()));
        {
            let mut env = global.borrow_mut();
            env.definir("altiro", Valor::Nativa(Nativa::Altiro), false)
                .expect("builtin 'altiro' debe registrarse");
            env.definir("largo", Valor::Nativa(Nativa::Largo), false)
                .expect("builtin 'largo' debe registrarse");
            env.definir("cachar", Valor::Nativa(Nativa::Cachar), false)
                .expect("builtin 'cachar' debe registrarse");
            env.definir("pregunta", Valor::Nativa(Nativa::Pregunta), false)
                .expect("builtin 'pregunta' debe registrarse");
        }
        Interprete {
            global,
            salida: Rc::new(RefCell::new(Box::new(salida))),
        }
    }

    pub fn correr(&mut self, stmts: &[Stmt]) -> Result<Valor, PiolaError> {
        self.eval_stmts(stmts, Rc::clone(&self.global))
            .map_err(|e| match e {
                RuntimeError::Retorno(_) => PiolaError::Runtime {
                    mensaje: "'devolver' solo puede usarse dentro de una pega papito.".to_string(),
                },
                RuntimeError::Cortala => PiolaError::Runtime {
                    mensaje: "'cortala' solo tiene sentido dentro de un bucle, po.".to_string(),
                },
                RuntimeError::Sigue => PiolaError::Runtime {
                    mensaje: "'sigue' solo tiene sentido dentro de un bucle, po.".to_string(),
                },
                _ => PiolaError::Runtime {
                    mensaje: e.to_string(),
                },
            })
    }

    fn eval_stmts(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Entorno>>,
    ) -> Result<Valor, RuntimeError> {
        let mut last = Valor::Nada;
        for stmt in stmts {
            last = self.eval_stmt(stmt, Rc::clone(&env))?;
        }
        Ok(last)
    }

    fn eval_stmt(&mut self, stmt: &Stmt, env: Rc<RefCell<Entorno>>) -> Result<Valor, RuntimeError> {
        match stmt {
            Stmt::Expresion(expr) => self.eval_expr(expr, env),

            Stmt::DeclWea {
                nombre,
                valor,
                es_duro,
            } => {
                let v = self.eval_expr(valor, Rc::clone(&env))?;
                env.borrow_mut().definir(nombre, v, *es_duro)?;
                Ok(Valor::Nada)
            }

            Stmt::DeclPega {
                nombre,
                params,
                cuerpo,
            } => {
                let funcion = Valor::Funcion {
                    params: params.clone(),
                    cuerpo: cuerpo.clone(),
                    entorno: Rc::clone(&env),
                };
                env.borrow_mut().definir(nombre, funcion, false)?;
                Ok(Valor::Nada)
            }

            Stmt::Cachai {
                cond,
                entonces,
                si_no,
            } => {
                let condicion = self.eval_expr(cond, Rc::clone(&env))?;
                if condicion.es_verdadero() {
                    self.eval_stmts(entonces, nuevo_scope(&env))
                } else if let Some(rama_no) = si_no {
                    self.eval_stmts(rama_no, nuevo_scope(&env))
                } else {
                    Ok(Valor::Nada)
                }
            }

            Stmt::Mientras { cond, cuerpo } => {
                loop {
                    let condicion = self.eval_expr(cond, Rc::clone(&env))?;
                    if !condicion.es_verdadero() {
                        break;
                    }
                    exec_body_loop!(self, cuerpo, nuevo_scope(&env));
                }
                Ok(Valor::Nada)
            }

            Stmt::Para {
                var,
                iterable,
                cuerpo,
            } => {
                let coleccion = self.eval_expr(iterable, Rc::clone(&env))?;
                match coleccion {
                    Valor::Lista(items) => {
                        let snapshot: Vec<Valor> = items.borrow().clone();
                        for item in snapshot {
                            let child = nuevo_scope(&env);
                            child.borrow_mut().definir(var, item, false)?;
                            exec_body_loop!(self, cuerpo, child);
                        }
                    }
                    Valor::Texto(s) => {
                        for ch in s.chars() {
                            let child = nuevo_scope(&env);
                            child
                                .borrow_mut()
                                .definir(var, Valor::Texto(ch.to_string()), false)?;
                            exec_body_loop!(self, cuerpo, child);
                        }
                    }
                    other => {
                        return Err(RuntimeError::TipoInvalido(format!(
                            "No podí iterar sobre un '{}', solo listas y textos.",
                            other.tipo_nombre()
                        )));
                    }
                }
                Ok(Valor::Nada)
            }

            Stmt::Ojo {
                cuerpo,
                error_var,
                manejo,
            } => {
                match self.eval_stmts(cuerpo, nuevo_scope(&env)) {
                    Ok(v) => Ok(v),
                    // Señales de control flow pasan de largo — ojo no las captura.
                    // Sin esto, `devolver` dentro de un `ojo` sería atrapado por `cago`.
                    Err(e @ RuntimeError::Retorno(_)) => Err(e),
                    Err(e @ RuntimeError::Cortala) => Err(e),
                    Err(e @ RuntimeError::Sigue) => Err(e),
                    // Solo errores reales llegan al cago
                    Err(e) => {
                        let child = nuevo_scope(&env);
                        child.borrow_mut().definir(
                            error_var,
                            Valor::Texto(e.to_string()),
                            false,
                        )?;
                        self.eval_stmts(manejo, child)
                    }
                }
            }

            Stmt::Devolver { valor } => {
                let v = self.eval_expr(valor, env)?;
                Err(RuntimeError::Retorno(v))
            }

            Stmt::Cortala => Err(RuntimeError::Cortala),
            Stmt::Sigue => Err(RuntimeError::Sigue),
        }
    }

    fn eval_expr(&mut self, expr: &Expr, env: Rc<RefCell<Entorno>>) -> Result<Valor, RuntimeError> {
        match expr {
            Expr::Numero(n) => Ok(Valor::Numero(*n)),
            Expr::Texto(s) => Ok(Valor::Texto(s.clone())),
            Expr::Booleano(b) => Ok(Valor::Booleano(*b)),
            Expr::Nada => Ok(Valor::Nada),

            Expr::Ident(nombre, _span) => env
                .borrow()
                .obtener(nombre)
                .ok_or_else(|| RuntimeError::VarNoDefinida(nombre.clone())),

            Expr::Asignacion { nombre, valor, .. } => {
                let v = self.eval_expr(valor, Rc::clone(&env))?;
                env.borrow_mut().asignar(nombre, v.clone())?;
                Ok(v)
            }

            Expr::Unario { op, expr, .. } => {
                let v = self.eval_expr(expr, env)?;
                match op {
                    OpUn::No => Ok(Valor::Booleano(!v.es_verdadero())),
                    OpUn::Neg => match v {
                        Valor::Numero(n) => Ok(Valor::Numero(-n)),
                        other => Err(RuntimeError::TipoInvalido(format!(
                            "No podí negar un '{}'.",
                            other.tipo_nombre()
                        ))),
                    },
                }
            }

            Expr::Binario { izq, op, der, .. } => {
                // Short-circuit antes de evaluar ambos lados
                match op {
                    OpBin::Y => {
                        let lhs = self.eval_expr(izq, Rc::clone(&env))?;
                        if !lhs.es_verdadero() {
                            return Ok(Valor::Booleano(false));
                        }
                        let rhs = self.eval_expr(der, env)?;
                        return Ok(Valor::Booleano(rhs.es_verdadero()));
                    }
                    OpBin::O => {
                        let lhs = self.eval_expr(izq, Rc::clone(&env))?;
                        if lhs.es_verdadero() {
                            return Ok(Valor::Booleano(true));
                        }
                        let rhs = self.eval_expr(der, env)?;
                        return Ok(Valor::Booleano(rhs.es_verdadero()));
                    }
                    _ => {}
                }
                let lhs = self.eval_expr(izq, Rc::clone(&env))?;
                let rhs = self.eval_expr(der, env)?;
                self.eval_binario(op, lhs, rhs)
            }

            Expr::Llamada { callee, args, .. } => {
                let func = self.eval_expr(callee, Rc::clone(&env))?;
                let mut evaluated_args = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, Rc::clone(&env))?);
                }
                self.llamar(func, evaluated_args)
            }

            Expr::Indice { objeto, indice, .. } => {
                let obj = self.eval_expr(objeto, Rc::clone(&env))?;
                let idx = self.eval_expr(indice, env)?;
                self.eval_indice(obj, idx)
            }

            Expr::Lista(items, _) => {
                let mut vals = Vec::with_capacity(items.len());
                for item in items {
                    vals.push(self.eval_expr(item, Rc::clone(&env))?);
                }
                Ok(Valor::Lista(Rc::new(RefCell::new(vals))))
            }

            Expr::Mapa(pairs, _) => {
                let mut map = HashMap::new();
                for (key_expr, val_expr) in pairs {
                    let key = self.eval_expr(key_expr, Rc::clone(&env))?;
                    let val = self.eval_expr(val_expr, Rc::clone(&env))?;
                    let key_str = match key {
                        Valor::Texto(s) => s,
                        other => other.to_string(),
                    };
                    map.insert(key_str, val);
                }
                Ok(Valor::Mapa(Rc::new(RefCell::new(map))))
            }
        }
    }

    fn eval_binario(&self, op: &OpBin, lhs: Valor, rhs: Valor) -> Result<Valor, RuntimeError> {
        match op {
            OpBin::Suma => match (&lhs, &rhs) {
                (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(a + b)),
                (Valor::Texto(a), Valor::Texto(b)) => Ok(Valor::Texto(format!("{a}{b}"))),
                // Coerción texto + número: el número se convierte a texto
                (Valor::Texto(a), Valor::Numero(b)) => {
                    Ok(Valor::Texto(format!("{a}{}", format_num(*b))))
                }
                (Valor::Numero(a), Valor::Texto(b)) => {
                    Ok(Valor::Texto(format!("{}{b}", format_num(*a))))
                }
                _ => Err(RuntimeError::TipoInvalido(format!(
                    "No podi sumar un '{}' con un '{}' pedazo de saco wea.",
                    lhs.tipo_nombre(),
                    rhs.tipo_nombre()
                ))),
            },
            OpBin::Resta => numeric_op(lhs, rhs, "-", |a, b| a - b),
            OpBin::Mul => numeric_op(lhs, rhs, "*", |a, b| a * b),
            OpBin::Div => numeric_op_checked(lhs, rhs, "/", |a, b| a / b), // ← unificado
            OpBin::Mod => numeric_op_checked(lhs, rhs, "%", |a, b| a % b), // ← unificado
            OpBin::Eq => Ok(Valor::Booleano(lhs == rhs)),
            OpBin::Neq => Ok(Valor::Booleano(lhs != rhs)),
            OpBin::Lt => compare_op(lhs, rhs, |a, b| a < b, |a, b| a < b),
            OpBin::Gt => compare_op(lhs, rhs, |a, b| a > b, |a, b| a > b),
            OpBin::Lte => compare_op(lhs, rhs, |a, b| a <= b, |a, b| a <= b),
            OpBin::Gte => compare_op(lhs, rhs, |a, b| a >= b, |a, b| a >= b),
            OpBin::Y | OpBin::O => unreachable!(), // manejados con short-circuit arriba
        }
    }

    fn llamar(&mut self, func: Valor, args: Vec<Valor>) -> Result<Valor, RuntimeError> {
        match func {
            Valor::Nativa(nativa) => self.llamar_nativa(nativa, args),
            Valor::Funcion {
                params,
                cuerpo,
                entorno,
            } => {
                if args.len() != params.len() {
                    return Err(RuntimeError::NumArgInvalido(params.len(), args.len()));
                }
                let call_env = nuevo_scope(&entorno);
                for (param, arg) in params.iter().zip(args) {
                    call_env.borrow_mut().definir(param, arg, false)?;
                }
                match self.eval_stmts(&cuerpo, call_env) {
                    Ok(v) => Ok(v),
                    Err(RuntimeError::Retorno(v)) => Ok(v), // ← atrapa devolver
                    Err(e) => Err(e),
                }
            }
            other => Err(RuntimeError::NoLlamable(other.tipo_nombre().to_string())),
        }
    }

    fn eval_indice(&self, obj: Valor, idx: Valor) -> Result<Valor, RuntimeError> {
        match obj {
            Valor::Lista(items) => {
                let items = items.borrow();
                let i = numeric_idx(&idx, "lista")?;
                let real_idx = resolver_indice(i, items.len())?; // ← unificado
                Ok(items[real_idx].clone())
            }
            Valor::Mapa(map) => {
                let map = map.borrow();
                let key = match idx {
                    Valor::Texto(s) => s,
                    other => other.to_string(),
                };
                map.get(&key)
                    .cloned()
                    .ok_or(RuntimeError::ClaveInexistente(key))
            }
            Valor::Texto(s) => {
                let chars = s.chars().collect::<Vec<_>>();
                let i = numeric_idx(&idx, "texto")?;
                let real_idx = resolver_indice(i, chars.len())?; // ← unificado
                Ok(Valor::Texto(chars[real_idx].to_string()))
            }
            other => Err(RuntimeError::TipoInvalido(format!(
                "No podí indexar un '{}'.",
                other.tipo_nombre()
            ))),
        }
    }

    fn llamar_nativa(&mut self, nativa: Nativa, args: Vec<Valor>) -> Result<Valor, RuntimeError> {
        match nativa {
            Nativa::Altiro => self.builtin_altiro(args),
            Nativa::Largo => builtin_largo(args),
            Nativa::Cachar => builtin_cachar(args),
            Nativa::Pregunta => builtin_pregunta(args),
        }
    }

    fn builtin_altiro(&mut self, args: Vec<Valor>) -> Result<Valor, RuntimeError> {
        let linea = args
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(self.salida.borrow_mut(), "{linea}")
            .map_err(|e| RuntimeError::TipoInvalido(format!("Error escribiendo output: {e}")))?;
        Ok(Valor::Nada)
    }
}

fn numeric_op(
    lhs: Valor,
    rhs: Valor,
    op_sym: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<Valor, RuntimeError> {
    match (lhs, rhs) {
        (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(f(a, b))),
        (l, r) => Err(RuntimeError::TipoInvalido(format!(
            "No podi usar '{op_sym}' entre un '{}' y un '{}' pedazo de saco wea.",
            l.tipo_nombre(),
            r.tipo_nombre()
        ))),
    }
}

fn compare_op(
    lhs: Valor,
    rhs: Valor,
    f_num: impl Fn(f64, f64) -> bool,
    f_str: impl Fn(&str, &str) -> bool,
) -> Result<Valor, RuntimeError> {
    match (lhs, rhs) {
        (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Booleano(f_num(a, b))),
        (Valor::Texto(a), Valor::Texto(b)) => Ok(Valor::Booleano(f_str(&a, &b))),
        (l, r) => Err(RuntimeError::TipoInvalido(format!(
            "No podi comparar un '{}' con un '{}' pedazo de saco wea.",
            l.tipo_nombre(),
            r.tipo_nombre()
        ))),
    }
}

fn format_num(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

fn builtin_largo(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::NumArgInvalido(1, args.len()));
    }
    match &args[0] {
        Valor::Texto(s) => Ok(Valor::Numero(s.chars().count() as f64)),
        Valor::Lista(items) => Ok(Valor::Numero(items.borrow().len() as f64)),
        other => Err(RuntimeError::TipoInvalido(format!(
            "largo() solo funciona con texto o lista, no con '{}'.",
            other.tipo_nombre()
        ))),
    }
}

fn builtin_cachar(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::NumArgInvalido(1, args.len()));
    }
    Ok(Valor::Texto(args[0].tipo_nombre().to_string()))
}

fn builtin_pregunta(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::NumArgInvalido(1, args.len()));
    }
    print!("{}", args[0]);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| RuntimeError::TipoInvalido(format!("Error leyendo input: {e}")))?;
    Ok(Valor::Texto(
        input
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string(),
    ))
}
