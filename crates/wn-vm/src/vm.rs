//! Stack VM para bytecode de WN++.
//!
//! Ejecuta un [`Chunk`] instrucción por instrucción manteniendo un stack de
//! valores, un call stack de [`CallFrame`]s y un mapa de globales persistente.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io::{self, BufRead, BufReader, Cursor, Write},
    rc::Rc,
    sync::Arc,
};

use crate::native::NativeContext;
use crate::resolver::{ErrorResolucion, ModuleResolver, ModuloResuelto, NoopResolver};
use crate::{
    builtins,
    chunk::Chunk,
    compiler::compilar_modulo,
    opcode::OpCode,
    value::{NativeFn, ObjClosure, ObjFunction, ObjModulo, ObjUpvalue, UpvalueState, Value},
};
use wn::{lexer::tokenizar_archivo, parser::parsear};
use wn_diagnostics::{SourceFile, WnDiagnostic};

#[derive(Debug, Clone, thiserror::Error)]
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

    #[error("Weon, no se puede dividir por cero.")]
    DivisionPorCero,

    #[error("'{0}' no es una pega papito.")]
    NoLlamable(String),

    #[error("La pega espera {esperados} argumento(s), le pasaste {recibidos}.")]
    NumArgInvalido { esperados: usize, recibidos: usize },

    #[error("Te fuiste al chancho, el índice {indice} no existe en la lista (largo: {largo}).")]
    IndiceInvalido { indice: i64, largo: usize },

    #[error("La clave '{clave}' no existe en el mapa papito.")]
    ClaveInexistente { clave: String },

    #[error("{0}")]
    TipoInvalido(String),

    #[error("No pude convertir {0:?} a número.")]
    TextoNoConvertibleANumero(String),

    #[error("pregunta() pidió más entrada que la provista.")]
    EntradaAgotada,

    #[error("El módulo '{0}' no existe. ¿Lo registraste con queri?")]
    ModuloNoEncontrado(String),

    #[error("El módulo '{modulo}' no tiene '{campo}'.")]
    CampoNoExisteEnModulo { modulo: String, campo: String },

    #[error("'{0}' no es un módulo.")]
    NoEsModulo(String),

    #[error("No pude resolver el módulo '{modulo}'.")]
    Resolucion {
        modulo: String,
        error: ErrorResolucion,
    },

    #[error("Importación circular: {0}")]
    ImportCircular(String),

    #[error("'{0}' ya está definido en este módulo.")]
    NombreEnUso(String),

    #[error("'{campo}' no es público en el módulo '{modulo}'.")]
    NoExportado { modulo: String, campo: String },

    #[error("El módulo '{modulo}' no tiene nada llamado '{campo}'.")]
    ExportInexistente {
        modulo: String,
        campo: String,
        disponibles: Vec<String>,
    },

    #[error("Falló la carga del módulo '{modulo}'.")]
    Carga {
        modulo: String,
        causa: Box<WnDiagnostic>,
    },

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
#[derive(Debug)]
pub struct CallFrame {
    pub closure: Rc<ObjClosure>,
    pub ip: usize,
    pub base_slot: usize,
    handlers: Vec<ExceptionHandler>,
    /// `Some` si este frame es la inicialización de un módulo de usuario.
    init: Option<InitModulo>,
}

/// Estado de una inicialización de módulo en curso (vive en el frame del módulo).
#[derive(Debug)]
struct InitModulo {
    /// Clave de la caché (ruta canónica del archivo).
    id: String,
    /// Ámbito de globales del módulo.
    ambito: usize,
    /// Path con el que se pidió: `utilidades::texto`.
    nombre: String,
    exportados: HashSet<String>,
    /// Ámbito y nombre donde se enlaza el módulo al terminar de inicializarse.
    destino: usize,
    binding: String,
    /// Dónde está el `queri` (para reportar errores del módulo en el importador).
    sitio_src: Arc<SourceFile>,
    sitio_span: (usize, usize),
}

/// Estado de un módulo en la caché. Un módulo cuya inicialización falla se
/// elimina de la caché, por eso no existe un estado "fallido".
#[derive(Debug, Clone)]
enum EstadoModulo {
    Cargando,
    Cargado(Value),
}

#[derive(Debug, Clone, Copy)]
struct ExceptionHandler {
    catch_ip: usize,
    stack_depth: usize,
    error_slot: u8,
}

enum StepAction {
    Continue,
    Return(Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GcKind {
    Texto,
    Lista,
    Mapa,
    Funcion,
    Closure,
    Iterador,
    Modulo,
}

#[derive(Debug, Clone, Copy)]
struct TrackedObject {
    bytes: usize,
    marked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GcStats {
    pub tracked_objects: usize,
    pub bytes_allocated: usize,
    pub next_gc: usize,
}

#[derive(Debug)]
struct GcArena {
    objects: HashMap<usize, TrackedObject>,
    bytes_allocated: usize,
    next_gc: usize,
    collecting: bool,
}

impl Default for GcArena {
    fn default() -> Self {
        Self {
            objects: HashMap::new(),
            bytes_allocated: 0,
            next_gc: 1024,
            collecting: false,
        }
    }
}

pub struct VM {
    stack: Vec<Value>,
    pub globals: HashMap<Rc<str>, Value>,
    pub frames: Vec<CallFrame>,
    open_upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    gc: GcArena,
    salida: Rc<RefCell<Box<dyn Write>>>,
    entrada: Rc<RefCell<Box<dyn BufRead>>>,
    resolver: Box<dyn ModuleResolver>,
    /// Ámbitos de globales de los módulos de usuario; el módulo con id `n` vive en `ambitos[n - 1]`.
    /// El ámbito 0 es `globals` (script de entrada o REPL).
    ambitos: Vec<HashMap<Rc<str>, Value>>,
    /// Caché de módulos de usuario por identidad normalizada.
    modulos: HashMap<String, EstadoModulo>,
    /// Funciones siempre disponibles, visibles desde cualquier módulo.
    builtins: HashMap<Rc<str>, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self::con_io(io::stdout(), BufReader::new(io::stdin()))
    }

    pub fn con_salida<W>(salida: W) -> Self
    where
        W: Write + 'static,
    {
        Self::con_io(salida, Cursor::new(Vec::<u8>::new()))
    }

    pub fn con_io<W, R>(salida: W, entrada: R) -> Self
    where
        W: Write + 'static,
        R: BufRead + 'static,
    {
        let mut vm = Self {
            stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            frames: Vec::with_capacity(64),
            open_upvalues: Vec::new(),
            gc: GcArena::default(),
            salida: Rc::new(RefCell::new(Box::new(salida))),
            entrada: Rc::new(RefCell::new(Box::new(entrada))),
            resolver: Box::new(NoopResolver),
            ambitos: Vec::new(),
            modulos: HashMap::new(),
            builtins: HashMap::new(),
        };
        vm.registrar_nativas();
        vm
    }

    /// Crea un VM con un resolver de módulos personalizado.
    /// `wn-cli` usa esto para conectar stdlib + módulos de usuario.
    pub fn con_resolver<W, R>(salida: W, entrada: R, resolver: Box<dyn ModuleResolver>) -> Self
    where
        W: Write + 'static,
        R: BufRead + 'static,
    {
        let mut vm = Self::con_io(salida, entrada);
        vm.resolver = resolver;
        vm
    }

    /// Ejecuta un chunk de script raíz en un frame nuevo.
    pub fn run(&mut self, chunk: &Chunk) -> Result<Value, WnDiagnostic> {
        let funcion = Rc::new(ObjFunction {
            chunk: chunk.clone(),
            aridad: 0,
            nombre: chunk.name.clone(),
            upvalues: Vec::new(),
        });
        let closure = Rc::new(ObjClosure {
            funcion,
            ambito: 0,
            upvalues: RefCell::new(Vec::new()),
        });
        self.track_value(&Value::Closure(closure.clone()));
        self.frames.push(CallFrame {
            closure,
            ip: 0,
            base_slot: self.stack.len(),
            handlers: Vec::new(),
            init: None,
        });
        self.execute()
    }

    fn execute(&mut self) -> Result<Value, WnDiagnostic> {
        loop {
            let frame_idx = self.frames.len().saturating_sub(1);
            let (op, base_slot, op_offset) = {
                let frame = self.frames.last_mut().expect("frame activo");
                let chunk = &frame.closure.funcion.chunk;
                let op_offset = frame.ip;
                let byte = chunk.code[frame.ip];
                frame.ip += 1;
                if let Some(op) = OpCode::from_byte(byte) {
                    (op, frame.base_slot, op_offset)
                } else {
                    let source = chunk.source.clone();
                    let span = chunk.spans[op_offset].clone();
                    return Err(WnDiagnostic::runtime(
                        &source,
                        source.span(span.start, span.end),
                        VmError::OpcodeInvalido(byte).to_string(),
                    ));
                }
            };

            let step: Result<StepAction, VmError> = (|| match op {
                OpCode::Constante => {
                    let idx = self.read_u16_operand(frame_idx) as usize;
                    let valor = self.frames[frame_idx].closure.funcion.chunk.constants[idx].clone();
                    self.push(valor);
                    Ok(StepAction::Continue)
                }
                OpCode::Nada => {
                    self.push(Value::Nada);
                    Ok(StepAction::Continue)
                }
                OpCode::Verdad => {
                    self.push(Value::Booleano(true));
                    Ok(StepAction::Continue)
                }
                OpCode::Falso => {
                    self.push(Value::Booleano(false));
                    Ok(StepAction::Continue)
                }
                OpCode::Pop => {
                    self.pop()?;
                    Ok(StepAction::Continue)
                }
                OpCode::DefinirGlobal => {
                    let nombre = self.read_nombre_constante(frame_idx);
                    let valor = self.pop()?;
                    let ambito = self.frames[frame_idx].closure.ambito;
                    self.globals_de_mut(ambito).insert(nombre, valor);
                    Ok(StepAction::Continue)
                }
                OpCode::ObtenerGlobal => {
                    let nombre = self.read_nombre_constante(frame_idx);
                    let ambito = self.frames[frame_idx].closure.ambito;
                    let valor = self
                        .buscar_global(ambito, &nombre)
                        .ok_or_else(|| VmError::VarNoDefinida(nombre.to_string()))?
                        .clone();
                    self.push(valor);
                    Ok(StepAction::Continue)
                }
                OpCode::Importar => {
                    let path_idx = self.read_u16_operand(frame_idx) as usize;
                    let name_idx = self.read_u16_operand(frame_idx) as usize;

                    let (path_str, name) = {
                        let chunk = &self.frames[frame_idx].closure.funcion.chunk;
                        let path_str = match &chunk.constants[path_idx] {
                            Value::Texto(s) => s.clone(),
                            _ => unreachable!("Importar: path_idx no apunta a Texto"),
                        };
                        let name = match &chunk.constants[name_idx] {
                            Value::Texto(s) => s.to_string(),
                            _ => unreachable!("Importar: name_idx no apunta a Texto"),
                        };
                        (path_str, name)
                    };

                    self.importar(frame_idx, op_offset, &path_str, &name)?;
                    Ok(StepAction::Continue)
                }
                OpCode::ObtenerPath => {
                    let path_idx = self.read_u16_operand(frame_idx) as usize;

                    let path_str = {
                        let chunk = &self.frames[frame_idx].closure.funcion.chunk;
                        match &chunk.constants[path_idx] {
                            Value::Texto(s) => s.clone(),
                            _ => unreachable!("ObtenerPath: path_idx no apunta a Texto"),
                        }
                    };

                    // El penúltimo segmento es el nombre con el que se vinculó el módulo.
                    // `texto::dividir` = binding="texto",    campo="dividir"
                    // `math::vectores::rotar` = binding="vectores", campo="rotar"
                    let partes: Vec<&str> = path_str.split("::").collect();
                    if partes.len() < 2 {
                        return Err(VmError::TipoInvalido(format!(
                            "'{path_str}' no es un path calificado."
                        )));
                    }
                    let (campo, modulo_partes) = partes.split_last().unwrap();
                    let binding = modulo_partes.last().unwrap();

                    let ambito = self.frames[frame_idx].closure.ambito;
                    let modulo = self
                        .buscar_global(ambito, binding)
                        .ok_or_else(|| VmError::VarNoDefinida(binding.to_string()))?
                        .clone();

                    match modulo {
                        Value::Modulo(ref map) => {
                            let valor = map
                                .get(*campo)
                                .ok_or_else(|| VmError::CampoNoExisteEnModulo {
                                    modulo: binding.to_string(),
                                    campo: campo.to_string(),
                                })?
                                .clone();
                            self.push(valor);
                            Ok(StepAction::Continue)
                        }
                        Value::ModuloUsuario(ref modulo) => {
                            let valor = self.leer_exportado(modulo, binding, campo)?;
                            self.push(valor);
                            Ok(StepAction::Continue)
                        }
                        other => Err(VmError::NoEsModulo(other.tipo_nombre().to_string())),
                    }
                }

                OpCode::AsignarGlobal => {
                    let nombre = self.read_nombre_constante(frame_idx);
                    let valor = self.peek()?.clone();
                    let ambito = self.frames[frame_idx].closure.ambito;
                    if let Some(slot) = self.globals_de_mut(ambito).get_mut(&nombre) {
                        *slot = valor;
                        Ok(StepAction::Continue)
                    } else {
                        Err(VmError::VarNoDefinida(nombre.to_string()))
                    }
                }
                OpCode::ObtenerLocal => {
                    let slot = self.read_byte_operand(frame_idx) as usize;
                    let valor = self.stack[base_slot + slot].clone();
                    self.push(valor);
                    Ok(StepAction::Continue)
                }
                OpCode::AsignarLocal => {
                    let slot = self.read_byte_operand(frame_idx) as usize;
                    let valor = self.peek()?.clone();
                    self.stack[base_slot + slot] = valor;
                    Ok(StepAction::Continue)
                }
                OpCode::ObtenerUpvalue => {
                    let slot = self.read_byte_operand(frame_idx) as usize;
                    let upvalue = self.frames[frame_idx].closure.upvalues.borrow()[slot].clone();
                    let valor = self.leer_upvalue(&upvalue);
                    self.push(valor);
                    Ok(StepAction::Continue)
                }
                OpCode::AsignarUpvalue => {
                    let slot = self.read_byte_operand(frame_idx) as usize;
                    let valor = self.peek()?.clone();
                    let upvalue = self.frames[frame_idx].closure.upvalues.borrow()[slot].clone();
                    self.escribir_upvalue(&upvalue, valor);
                    Ok(StepAction::Continue)
                }
                OpCode::CerrarUpvalue => {
                    let slot = self.stack.len() - 1;
                    self.close_upvalues(slot);
                    self.pop()?;
                    Ok(StepAction::Continue)
                }
                OpCode::Suma => self.op_suma().map(|_| StepAction::Continue),
                OpCode::Resta => self
                    .op_binaria_numerica("-", |a, b| a - b)
                    .map(|_| StepAction::Continue),
                OpCode::Mul => self
                    .op_binaria_numerica("*", |a, b| a * b)
                    .map(|_| StepAction::Continue),
                OpCode::Div => self.op_div().map(|_| StepAction::Continue),
                OpCode::Mod => self.op_mod().map(|_| StepAction::Continue),
                OpCode::Neg => {
                    let valor = self.pop()?;
                    match valor {
                        Value::Numero(n) => {
                            self.push(Value::Numero(-n));
                            Ok(StepAction::Continue)
                        }
                        other => Err(VmError::NegacionInvalida(other.tipo_nombre())),
                    }
                }
                OpCode::No => {
                    let valor = self.pop()?;
                    self.push(Value::Booleano(!valor.es_verdadero()));
                    Ok(StepAction::Continue)
                }
                OpCode::Eq => {
                    let der = self.pop()?;
                    let izq = self.pop()?;
                    self.push(Value::Booleano(izq == der));
                    Ok(StepAction::Continue)
                }
                OpCode::Neq => {
                    let der = self.pop()?;
                    let izq = self.pop()?;
                    self.push(Value::Booleano(izq != der));
                    Ok(StepAction::Continue)
                }
                OpCode::Lt => self
                    .op_comparacion("<", |a, b| a < b, |a, b| a < b)
                    .map(|_| StepAction::Continue),
                OpCode::Gt => self
                    .op_comparacion(">", |a, b| a > b, |a, b| a > b)
                    .map(|_| StepAction::Continue),
                OpCode::Lte => self
                    .op_comparacion("<=", |a, b| a <= b, |a, b| a <= b)
                    .map(|_| StepAction::Continue),
                OpCode::Gte => self
                    .op_comparacion(">=", |a, b| a >= b, |a, b| a >= b)
                    .map(|_| StepAction::Continue),
                OpCode::Saltar => {
                    let offset = self.read_u16_operand(frame_idx) as usize;
                    self.frames[frame_idx].ip += offset;
                    Ok(StepAction::Continue)
                }
                OpCode::SaltarSiFalso => {
                    let offset = self.read_u16_operand(frame_idx) as usize;
                    if !self.peek()?.es_verdadero() {
                        self.frames[frame_idx].ip += offset;
                    }
                    Ok(StepAction::Continue)
                }
                OpCode::Loop => {
                    let offset = self.read_u16_operand(frame_idx) as usize;
                    self.frames[frame_idx].ip -= offset;
                    Ok(StepAction::Continue)
                }
                OpCode::PushHandler => {
                    let jump = self.read_u16_operand(frame_idx) as usize;
                    let error_slot = self.read_byte_operand(frame_idx);
                    let catch_ip = self.frames[frame_idx].ip + jump;
                    self.frames[frame_idx].handlers.push(ExceptionHandler {
                        catch_ip,
                        stack_depth: self.stack.len(),
                        error_slot,
                    });
                    Ok(StepAction::Continue)
                }
                OpCode::PopHandler => {
                    self.frames[frame_idx].handlers.pop();
                    Ok(StepAction::Continue)
                }
                OpCode::Closure => {
                    let idx = self.read_u16_operand(frame_idx) as usize;
                    let funcion =
                        match self.frames[frame_idx].closure.funcion.chunk.constants[idx].clone() {
                            Value::Funcion(funcion) => funcion,
                            _ => panic!("constante de closure no es función"),
                        };
                    let mut captures = Vec::with_capacity(funcion.upvalues.len());
                    for _ in &funcion.upvalues {
                        let is_local = self.read_byte_operand(frame_idx) != 0;
                        let index = self.read_byte_operand(frame_idx);
                        let capture = if is_local {
                            self.capture_upvalue(base_slot + index as usize)
                        } else {
                            self.frames[frame_idx].closure.upvalues.borrow()[index as usize].clone()
                        };
                        captures.push(capture);
                    }
                    let ambito = self.frames[frame_idx].closure.ambito;
                    let closure = self.alloc_closure(funcion, ambito, captures);
                    self.push(closure);
                    Ok(StepAction::Continue)
                }
                OpCode::Llamar => {
                    let argc = self.read_byte_operand(frame_idx) as usize;
                    let callee_idx = self.stack.len() - 1 - argc;
                    let callee = self.stack[callee_idx].clone();
                    self.call_value(callee, argc)?;
                    Ok(StepAction::Continue)
                }
                OpCode::Devolver => Ok(StepAction::Return(self.pop()?)),
                OpCode::ConstruirLista => {
                    let n = self.read_u16_operand(frame_idx) as usize;
                    let mut items = Vec::with_capacity(n);
                    for _ in 0..n {
                        items.push(self.pop()?);
                    }
                    items.reverse();
                    let lista = self.alloc_list(items);
                    self.push(lista);
                    Ok(StepAction::Continue)
                }
                OpCode::ConstruirMapa => {
                    let n = self.read_u16_operand(frame_idx) as usize;
                    let mut map = HashMap::with_capacity(n);
                    for _ in 0..n {
                        let valor = self.pop()?;
                        let clave = self.pop()?;
                        map.insert(clave.a_clave_mapa(), valor);
                    }
                    let mapa = self.alloc_map(map);
                    self.push(mapa);
                    Ok(StepAction::Continue)
                }
                OpCode::IterInit => self.op_iter_init().map(|_| StepAction::Continue),
                OpCode::IterNext => {
                    let slot = self.read_byte_operand(frame_idx);
                    self.op_iter_next(base_slot, slot)
                        .map(|_| StepAction::Continue)
                }
                OpCode::ObtenerIndice => self.op_obtener_indice().map(|_| StepAction::Continue),
                OpCode::AsignarIndice => self.op_asignar_indice().map(|_| StepAction::Continue),
                OpCode::Lorea => unreachable!("lorea corre como nativa, no como opcode dedicado"),
                OpCode::RetornarNada => Ok(StepAction::Return(Value::Nada)),
            })();

            match step {
                Ok(StepAction::Continue) => {}
                Ok(StepAction::Return(resultado)) => {
                    if let Some(resultado) = self.finish_return(resultado) {
                        return Ok(resultado);
                    }
                }
                Err(err) => {
                    let diagnostic = self.vm_error_to_diagnostic(frame_idx, op_offset, &err);
                    // El mensaje que ve `ojo ... cago` es el del error original; el
                    // envoltorio con la cadena de imports es solo para el reporte final.
                    let mensaje = diagnostic.to_string();
                    let diagnostic = self.envolver_error_de_modulo(diagnostic);
                    if !self.handle_error(mensaje) {
                        return Err(diagnostic);
                    }
                }
            }
        }
    }

    /// Registra el archivo de entrada en la caché para que `queri` del propio
    /// archivo principal se detecte como ciclo en vez de ejecutarlo dos veces.
    pub fn marcar_entrada(&mut self, id: impl Into<String>) {
        self.modulos.insert(id.into(), EstadoModulo::Cargando);
    }

    fn globals_de_mut(&mut self, ambito: usize) -> &mut HashMap<Rc<str>, Value> {
        if ambito == 0 {
            &mut self.globals
        } else {
            &mut self.ambitos[ambito - 1]
        }
    }

    /// Busca un global en el ámbito dado. Los módulos solo ven sus propios
    /// globales y las builtins; nunca los del script de entrada ni los de otros módulos.
    fn buscar_global(&self, ambito: usize, nombre: &str) -> Option<&Value> {
        if ambito == 0 {
            self.globals.get(nombre)
        } else {
            self.ambitos[ambito - 1]
                .get(nombre)
                .or_else(|| self.builtins.get(nombre))
        }
    }

    /// ¿Se puede enlazar `binding` en `destino`? Rebindear un módulo ya importado
    /// o tapar una builtin está permitido; pisar cualquier otra cosa no.
    fn binding_libre(&self, destino: usize, binding: &str, nuevo: Option<&Value>) -> bool {
        let previo = if destino == 0 {
            self.globals.get(binding)
        } else {
            self.ambitos[destino - 1].get(binding)
        };
        match (previo, nuevo) {
            (None, _) => true,
            (Some(Value::Nativa(_)), _) => self.builtins.contains_key(binding),
            (Some(Value::Modulo(_)), Some(Value::Modulo(_))) => true,
            (Some(Value::ModuloUsuario(a)), Some(Value::ModuloUsuario(b))) => a.id == b.id,
            _ => false,
        }
    }

    fn importar(
        &mut self,
        frame_idx: usize,
        op_offset: usize,
        path_str: &str,
        binding: &str,
    ) -> Result<(), VmError> {
        let destino = self.frames[frame_idx].closure.ambito;
        let partes: Vec<&str> = path_str.split("::").collect();
        let resuelto = self
            .resolver
            .resolver(&partes)
            .map_err(|error| VmError::Resolucion {
                modulo: path_str.to_string(),
                error,
            })?
            .ok_or_else(|| VmError::ModuloNoEncontrado(path_str.to_string()))?;

        match resuelto {
            ModuloResuelto::Nativo(modulo) => {
                if !self.binding_libre(destino, binding, Some(&modulo)) {
                    return Err(VmError::NombreEnUso(binding.to_string()));
                }
                self.track_value(&modulo);
                self.globals_de_mut(destino)
                    .insert(Rc::from(binding), modulo);
                Ok(())
            }
            ModuloResuelto::Fuente { id, source } => match self.modulos.get(&id).cloned() {
                Some(EstadoModulo::Cargado(modulo)) => {
                    if !self.binding_libre(destino, binding, Some(&modulo)) {
                        return Err(VmError::NombreEnUso(binding.to_string()));
                    }
                    self.globals_de_mut(destino)
                        .insert(Rc::from(binding), modulo);
                    Ok(())
                }
                Some(EstadoModulo::Cargando) => {
                    Err(VmError::ImportCircular(self.cadena_imports(Some(path_str))))
                }
                None => {
                    if !self.binding_libre(destino, binding, None) {
                        return Err(VmError::NombreEnUso(binding.to_string()));
                    }
                    self.cargar_modulo(frame_idx, op_offset, id, source, path_str, binding)
                }
            },
        }
    }

    /// Compila el módulo, le crea su ámbito y empuja su frame de inicialización.
    /// el loop principal lo ejecuta como cualquier otro frame.
    fn cargar_modulo(
        &mut self,
        frame_idx: usize,
        op_offset: usize,
        id: String,
        source: Arc<SourceFile>,
        path_str: &str,
        binding: &str,
    ) -> Result<(), VmError> {
        let carga = |causa: WnDiagnostic| VmError::Carga {
            modulo: path_str.to_string(),
            causa: Box::new(causa),
        };
        let tokens = tokenizar_archivo(source.src(), source.filename()).map_err(carga)?;
        let stmts = parsear(tokens, source.src(), source.filename()).map_err(carga)?;
        let (chunk, exportados) = compilar_modulo(&stmts, source.clone()).map_err(carga)?;

        let (destino, sitio_src, sitio_span) = {
            let frame = &self.frames[frame_idx];
            let chunk = &frame.closure.funcion.chunk;
            let span = &chunk.spans[op_offset];
            (
                frame.closure.ambito,
                chunk.source.clone(),
                (span.start, span.end),
            )
        };

        self.ambitos.push(HashMap::new());
        let ambito = self.ambitos.len();
        self.modulos.insert(id.clone(), EstadoModulo::Cargando);

        let funcion = Rc::new(ObjFunction {
            chunk,
            aridad: 0,
            nombre: path_str.to_string(),
            upvalues: Vec::new(),
        });
        let closure = Rc::new(ObjClosure {
            funcion,
            ambito,
            upvalues: RefCell::new(Vec::new()),
        });
        self.track_value(&Value::Closure(closure.clone()));
        self.frames.push(CallFrame {
            closure,
            ip: 0,
            base_slot: self.stack.len(),
            handlers: Vec::new(),
            init: Some(InitModulo {
                id,
                ambito,
                nombre: path_str.to_string(),
                exportados: exportados.into_iter().collect(),
                destino,
                binding: binding.to_string(),
                sitio_src,
                sitio_span,
            }),
        });
        Ok(())
    }

    /// El frame del módulo terminó bien, queda cargado en la caché y se enlaza
    /// en el ámbito del importador.
    fn completar_modulo(&mut self, init: InitModulo) {
        let modulo = Value::ModuloUsuario(Rc::new(ObjModulo {
            id: init.ambito,
            nombre: init.nombre,
            exportados: init.exportados,
        }));
        self.modulos
            .insert(init.id, EstadoModulo::Cargado(modulo.clone()));
        self.globals_de_mut(init.destino)
            .insert(Rc::from(init.binding.as_str()), modulo);
    }

    /// La inicialización del módulo falló, no queda en la caché ni con estado parcial.
    fn abandonar_modulo(&mut self, init: &InitModulo) {
        self.modulos.remove(&init.id);
        self.ambitos[init.ambito - 1].clear();
    }

    fn leer_exportado(
        &self,
        modulo: &ObjModulo,
        binding: &str,
        campo: &str,
    ) -> Result<Value, VmError> {
        let ambito = &self.ambitos[modulo.id - 1];
        if modulo.exportados.contains(campo)
            && let Some(valor) = ambito.get(campo)
        {
            return Ok(valor.clone());
        }
        if ambito.contains_key(campo) {
            return Err(VmError::NoExportado {
                modulo: binding.to_string(),
                campo: campo.to_string(),
            });
        }
        let mut disponibles: Vec<String> = modulo.exportados.iter().cloned().collect();
        disponibles.sort();
        Err(VmError::ExportInexistente {
            modulo: binding.to_string(),
            campo: campo.to_string(),
            disponibles,
        })
    }

    fn cadena_imports(&self, extra: Option<&str>) -> String {
        let archivo = |nombre: &str| format!("{}.cl", nombre.replace("::", "/"));
        let origen = self
            .frames
            .first()
            .map(|frame| {
                let filename = frame.closure.funcion.chunk.source.filename();
                std::path::Path::new(filename).file_name().map_or_else(
                    || filename.to_string(),
                    |n| n.to_string_lossy().into_owned(),
                )
            })
            .unwrap_or_else(|| "<principal>".to_string());
        let mut cadena = vec![origen];
        cadena.extend(
            self.frames
                .iter()
                .filter_map(|frame| frame.init.as_ref())
                .map(|init| archivo(&init.nombre)),
        );
        cadena.extend(extra.map(archivo));
        cadena.join(" → ")
    }

    /// Si el error ocurrió dentro de uno o más módulos en inicialización, lo envuelve
    /// con el sitio del `queri` más externo y la cadena de imports. El diagnóstico
    /// original (con su archivo y span) queda como causa.
    fn envolver_error_de_modulo(&self, diagnostic: WnDiagnostic) -> WnDiagnostic {
        let Some(externo) = self.frames.iter().find_map(|frame| frame.init.as_ref()) else {
            return diagnostic;
        };
        let (inicio, fin) = externo.sitio_span;
        WnDiagnostic::error_en_modulo(
            &externo.sitio_src,
            externo.sitio_src.span(inicio, fin),
            self.cadena_imports(None),
            diagnostic,
        )
    }

    fn registrar_nativas(&mut self) {
        for nativa in builtins::NATIVAS_CORE {
            self.globals
                .insert(Rc::from(nativa.nombre), Value::Nativa(*nativa));
            self.builtins
                .insert(Rc::from(nativa.nombre), Value::Nativa(*nativa));
        }
    }

    fn call_value(&mut self, callee: Value, argc: usize) -> Result<(), VmError> {
        match callee {
            Value::Closure(closure) => {
                if argc != closure.funcion.aridad {
                    return Err(VmError::NumArgInvalido {
                        esperados: closure.funcion.aridad,
                        recibidos: argc,
                    });
                }
                let base_slot = self.stack.len() - argc - 1;
                self.frames.push(CallFrame {
                    closure,
                    ip: 0,
                    base_slot,
                    handlers: Vec::new(),
                    init: None,
                });
                Ok(())
            }
            Value::Nativa(nativa) => {
                let result = self.call_native(nativa, argc)?;
                self.push(result);
                Ok(())
            }
            other => Err(VmError::NoLlamable(other.tipo_nombre().to_string())),
        }
    }

    fn call_native(&mut self, nativa: NativeFn, argc: usize) -> Result<Value, VmError> {
        if let Some(aridad) = nativa.aridad {
            self.expect_arity(nativa.nombre, aridad, argc)?;
        }
        let callee_idx = self.stack.len() - argc - 1;
        let args = self.stack.split_off(callee_idx + 1);
        self.stack.pop();

        let mut ctx = NativeContext {
            salida: &self.salida,
            entrada: &self.entrada,
        };
        let result = (nativa.func)(&mut ctx, &args)?;
        self.track_value(&result);
        Ok(result)
    }

    fn expect_arity(&self, _name: &str, esperados: usize, recibidos: usize) -> Result<(), VmError> {
        if esperados == recibidos {
            Ok(())
        } else {
            Err(VmError::NumArgInvalido {
                esperados,
                recibidos,
            })
        }
    }

    /// Expone contadores útiles para pruebas del GC.
    pub fn gc_stats(&self) -> GcStats {
        GcStats {
            tracked_objects: self.gc.objects.len(),
            bytes_allocated: self.gc.bytes_allocated,
            next_gc: self.gc.next_gc,
        }
    }

    /// Ejecuta un ciclo mark-and-sweep sobre los roots visibles del VM.
    pub fn collect_garbage(&mut self) {
        self.gc.collecting = true;
        let stack_roots = self.stack.clone();
        for value in &stack_roots {
            self.mark_value(value);
        }

        let global_roots = self
            .globals
            .values()
            .chain(self.ambitos.iter().flat_map(|ambito| ambito.values()))
            .cloned()
            .collect::<Vec<_>>();
        for value in &global_roots {
            self.mark_value(value);
        }

        let frame_roots = self
            .frames
            .iter()
            .map(|frame| Value::Closure(frame.closure.clone()))
            .collect::<Vec<_>>();
        for value in &frame_roots {
            self.mark_value(value);
        }

        let closed_upvalues = self
            .open_upvalues
            .iter()
            .filter_map(|upvalue| match &upvalue.borrow().state {
                UpvalueState::Cerrada(valor) => Some(valor.clone()),
                UpvalueState::Abierta(_) => None,
            })
            .collect::<Vec<_>>();
        for value in &closed_upvalues {
            self.mark_value(value);
        }

        self.sweep();
        self.gc.collecting = false;
    }

    fn track_value(&mut self, value: &Value) {
        match value {
            Value::Texto(texto) => self.track_ptr(
                Rc::as_ptr(texto) as *const () as usize,
                GcKind::Texto,
                texto.len(),
            ),
            Value::Lista(lista) => self.track_ptr(
                Rc::as_ptr(lista) as *const () as usize,
                GcKind::Lista,
                lista.borrow().len() * size_of::<Value>(),
            ),
            Value::Mapa(mapa) => self.track_ptr(
                Rc::as_ptr(mapa) as *const () as usize,
                GcKind::Mapa,
                mapa.borrow().len() * (size_of::<String>() + size_of::<Value>()),
            ),
            Value::Modulo(m) => self.track_ptr(
                Rc::as_ptr(m) as *const () as usize,
                GcKind::Modulo,
                m.len() * (size_of::<String>() + size_of::<Value>()),
            ),
            Value::Funcion(funcion) => self.track_ptr(
                Rc::as_ptr(funcion) as *const () as usize,
                GcKind::Funcion,
                funcion.chunk.code.len() + funcion.chunk.constants.len() * size_of::<Value>(),
            ),
            Value::Closure(closure) => {
                self.track_ptr(
                    Rc::as_ptr(closure) as *const () as usize,
                    GcKind::Closure,
                    closure.upvalues.borrow().len() * size_of::<usize>(),
                );
                self.track_value(&Value::Funcion(closure.funcion.clone()));
            }
            Value::Iterador(iter) => self.track_ptr(
                Rc::as_ptr(iter) as *const () as usize,
                GcKind::Iterador,
                std::mem::size_of_val(&*iter.borrow()),
            ),
            Value::Numero(_)
            | Value::Booleano(_)
            | Value::Nada
            | Value::Nativa(_)
            | Value::ModuloUsuario(_) => {}
        }
    }

    fn track_ptr(&mut self, key: usize, _kind: GcKind, bytes: usize) {
        if let std::collections::hash_map::Entry::Vacant(entry) = self.gc.objects.entry(key) {
            entry.insert(TrackedObject {
                bytes,
                marked: false,
            });
            self.gc.bytes_allocated += bytes.max(1);
            if !self.gc.collecting && self.gc.bytes_allocated > self.gc.next_gc {
                self.collect_garbage();
            }
        }
    }

    fn mark_value(&mut self, value: &Value) {
        self.track_value(value);
        match value {
            Value::Texto(texto) => {
                self.mark_ptr(Rc::as_ptr(texto) as *const () as usize);
            }
            Value::Lista(lista) => {
                let key = Rc::as_ptr(lista) as *const () as usize;
                if !self.mark_ptr(key) {
                    return;
                }
                let items = lista.borrow().clone();
                for item in &items {
                    self.mark_value(item);
                }
            }
            Value::Mapa(mapa) => {
                let key = Rc::as_ptr(mapa) as *const () as usize;
                if !self.mark_ptr(key) {
                    return;
                }
                let items = mapa.borrow().values().cloned().collect::<Vec<_>>();
                for item in &items {
                    self.mark_value(item);
                }
            }
            Value::Modulo(m) => {
                let key = Rc::as_ptr(m) as *const () as usize;
                if !self.mark_ptr(key) {
                    return;
                }
                let values: Vec<Value> = m.values().cloned().collect();
                for val in &values {
                    self.mark_value(val);
                }
            }
            Value::Funcion(funcion) => {
                let key = Rc::as_ptr(funcion) as *const () as usize;
                if !self.mark_ptr(key) {
                    return;
                }
                let constants = funcion.chunk.constants.clone();
                for constant in &constants {
                    self.mark_value(constant);
                }
            }
            Value::Closure(closure) => {
                let key = Rc::as_ptr(closure) as *const () as usize;
                if !self.mark_ptr(key) {
                    return;
                }
                self.mark_value(&Value::Funcion(closure.funcion.clone()));
                let upvalues = closure.upvalues.borrow().clone();
                for upvalue in upvalues {
                    if let UpvalueState::Cerrada(valor) = &upvalue.borrow().state {
                        self.mark_value(valor);
                    }
                }
            }
            Value::Iterador(iter) => {
                let key = Rc::as_ptr(iter) as *const () as usize;
                if !self.mark_ptr(key) {
                    return;
                }
                match &*iter.borrow() {
                    crate::value::ObjIterator::Lista { items, .. } => {
                        for item in items {
                            self.mark_value(item);
                        }
                    }
                    crate::value::ObjIterator::Texto { chars, .. } => {
                        let _ = chars;
                    }
                }
            }
            Value::Numero(_)
            | Value::Booleano(_)
            | Value::Nada
            | Value::Nativa(_)
            | Value::ModuloUsuario(_) => {}
        }
    }

    fn mark_ptr(&mut self, key: usize) -> bool {
        if let Some(obj) = self.gc.objects.get_mut(&key) {
            if obj.marked {
                return false;
            }
            obj.marked = true;
        }
        true
    }

    fn sweep(&mut self) {
        let mut alive_bytes = 0usize;
        self.gc.objects.retain(|_, obj| {
            if obj.marked {
                obj.marked = false;
                alive_bytes += obj.bytes.max(1);
                true
            } else {
                false
            }
        });
        self.gc.bytes_allocated = alive_bytes;
        self.gc.next_gc = (alive_bytes.max(1024)).saturating_mul(2);
    }

    fn alloc_text(&mut self, text: impl Into<String>) -> Value {
        let value = Value::from(text.into());
        self.track_value(&value);
        value
    }

    fn alloc_list(&mut self, items: Vec<Value>) -> Value {
        let value = Value::Lista(Rc::new(RefCell::new(items)));
        self.track_value(&value);
        value
    }

    fn alloc_map(&mut self, map: HashMap<String, Value>) -> Value {
        let value = Value::Mapa(Rc::new(RefCell::new(map)));
        self.track_value(&value);
        value
    }

    fn alloc_closure(
        &mut self,
        funcion: Rc<ObjFunction>,
        ambito: usize,
        upvalues: Vec<Rc<RefCell<ObjUpvalue>>>,
    ) -> Value {
        let value = Value::Closure(Rc::new(ObjClosure {
            funcion,
            ambito,
            upvalues: RefCell::new(upvalues),
        }));
        self.track_value(&value);
        value
    }

    fn alloc_iterator(&mut self, iter: crate::value::ObjIterator) -> Value {
        let value = Value::Iterador(Rc::new(RefCell::new(iter)));
        self.track_value(&value);
        value
    }

    fn capture_upvalue(&mut self, slot: usize) -> Rc<RefCell<ObjUpvalue>> {
        if let Some(existing) = self.open_upvalues.iter().find_map(|candidate| {
            let state = &candidate.borrow().state;
            match state {
                UpvalueState::Abierta(existing_slot) if *existing_slot == slot => {
                    Some(candidate.clone())
                }
                _ => None,
            }
        }) {
            return existing;
        }

        let upvalue = Rc::new(RefCell::new(ObjUpvalue {
            state: UpvalueState::Abierta(slot),
        }));
        self.open_upvalues.push(upvalue.clone());
        upvalue
    }

    fn close_upvalues(&mut self, last_slot: usize) {
        let mut still_open = Vec::with_capacity(self.open_upvalues.len());
        for upvalue in self.open_upvalues.drain(..) {
            let maybe_slot = match upvalue.borrow().state {
                UpvalueState::Abierta(slot) => Some(slot),
                UpvalueState::Cerrada(_) => None,
            };
            if let Some(slot) = maybe_slot {
                if slot >= last_slot {
                    let valor = self.stack[slot].clone();
                    upvalue.borrow_mut().state = UpvalueState::Cerrada(valor);
                } else {
                    still_open.push(upvalue);
                }
            }
        }
        self.open_upvalues = still_open;
    }

    fn leer_upvalue(&self, upvalue: &Rc<RefCell<ObjUpvalue>>) -> Value {
        match &upvalue.borrow().state {
            UpvalueState::Abierta(slot) => self.stack[*slot].clone(),
            UpvalueState::Cerrada(valor) => valor.clone(),
        }
    }

    fn escribir_upvalue(&mut self, upvalue: &Rc<RefCell<ObjUpvalue>>, valor: Value) {
        match &mut upvalue.borrow_mut().state {
            UpvalueState::Abierta(slot) => self.stack[*slot] = valor,
            UpvalueState::Cerrada(cerrada) => *cerrada = valor,
        }
    }

    fn op_suma(&mut self) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        let resultado = match (izq, der) {
            (Value::Numero(a), Value::Numero(b)) => Value::Numero(a + b),
            (Value::Texto(a), Value::Texto(b)) => self.alloc_text(format!("{a}{b}")),
            (Value::Texto(a), Value::Numero(b)) => {
                self.alloc_text(format!("{a}{}", Value::Numero(b)))
            }
            (Value::Numero(a), Value::Texto(b)) => {
                self.alloc_text(format!("{}{b}", Value::Numero(a)))
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
        f_num: impl Fn(f64, f64) -> bool,
        f_texto: impl Fn(&str, &str) -> bool,
    ) -> Result<(), VmError> {
        let der = self.pop()?;
        let izq = self.pop()?;
        match (izq, der) {
            (Value::Numero(a), Value::Numero(b)) => {
                self.push(Value::Booleano(f_num(a, b)));
                Ok(())
            }
            (Value::Texto(a), Value::Texto(b)) => {
                self.push(Value::Booleano(f_texto(&a, &b)));
                Ok(())
            }
            (izq, der) => Err(VmError::TiposIncompatibles {
                op,
                tipo_izq: izq.tipo_nombre(),
                tipo_der: der.tipo_nombre(),
            }),
        }
    }

    fn op_obtener_indice(&mut self) -> Result<(), VmError> {
        let indice = self.pop()?;
        let objeto = self.pop()?;
        match objeto {
            Value::Lista(items) => {
                let items = items.borrow();
                let idx = numeric_idx(&indice, "lista")?;
                let real = resolver_indice(idx, items.len())?;
                self.push(items[real].clone());
            }
            Value::Mapa(map) => {
                let clave = indice.a_clave_mapa();
                let valor =
                    map.borrow()
                        .get(&clave)
                        .cloned()
                        .ok_or_else(|| VmError::ClaveInexistente {
                            clave: clave.clone(),
                        })?;
                self.push(valor);
            }
            Value::Texto(texto) => {
                let chars = texto.chars().collect::<Vec<_>>();
                let idx = numeric_idx(&indice, "texto")?;
                let real = resolver_indice(idx, chars.len())?;
                let texto = self.alloc_text(chars[real].to_string());
                self.push(texto);
            }
            other => {
                return Err(VmError::TipoInvalido(format!(
                    "No podí indexar un '{}'.",
                    other.tipo_nombre()
                )));
            }
        }
        Ok(())
    }

    fn op_asignar_indice(&mut self) -> Result<(), VmError> {
        let valor = self.pop()?;
        let indice = self.pop()?;
        let objeto = self.pop()?;
        match objeto {
            Value::Lista(items) => {
                let mut items = items.borrow_mut();
                let idx = numeric_idx(&indice, "lista")?;
                let real = resolver_indice(idx, items.len())?;
                items[real] = valor.clone();
                self.push(valor);
            }
            Value::Mapa(map) => {
                map.borrow_mut()
                    .insert(indice.a_clave_mapa(), valor.clone());
                self.push(valor);
            }
            other => {
                return Err(VmError::TipoInvalido(format!(
                    "No podí asignar por índice sobre un '{}'.",
                    other.tipo_nombre()
                )));
            }
        }
        Ok(())
    }

    fn op_iter_init(&mut self) -> Result<(), VmError> {
        let coleccion = self.pop()?;
        let iter = match coleccion {
            Value::Lista(items) => self.alloc_iterator(crate::value::ObjIterator::Lista {
                items: items.borrow().clone(),
                index: 0,
            }),
            Value::Texto(texto) => self.alloc_iterator(crate::value::ObjIterator::Texto {
                chars: texto.chars().map(|ch| ch.to_string()).collect(),
                index: 0,
            }),
            other => {
                return Err(VmError::TipoInvalido(format!(
                    "No podí iterar sobre un '{}', solo listas y textos.",
                    other.tipo_nombre()
                )));
            }
        };
        self.push(iter);
        Ok(())
    }

    fn op_iter_next(&mut self, base_slot: usize, slot: u8) -> Result<(), VmError> {
        let iter = self.stack[base_slot + slot as usize].clone();
        match iter {
            Value::Iterador(iter) => match &mut *iter.borrow_mut() {
                crate::value::ObjIterator::Lista { items, index } => {
                    if let Some(valor) = items.get(*index).cloned() {
                        *index += 1;
                        self.push(valor);
                        self.push(Value::Booleano(true));
                    } else {
                        self.push(Value::Booleano(false));
                    }
                }
                crate::value::ObjIterator::Texto { chars, index } => {
                    if let Some(valor) = chars.get(*index).cloned() {
                        *index += 1;
                        let texto = self.alloc_text(valor);
                        self.push(texto);
                        self.push(Value::Booleano(true));
                    } else {
                        self.push(Value::Booleano(false));
                    }
                }
            },
            _ => panic!("slot de iteración no contiene iterador"),
        }
        Ok(())
    }

    #[inline]
    fn push(&mut self, valor: Value) {
        self.stack.push(valor);
    }

    #[inline]
    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    #[inline]
    fn peek(&self) -> Result<&Value, VmError> {
        self.stack.last().ok_or(VmError::StackUnderflow)
    }

    #[inline]
    fn read_byte_operand(&mut self, frame_idx: usize) -> u8 {
        let frame = &mut self.frames[frame_idx];
        let byte = frame.closure.funcion.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    #[inline]
    fn read_u16_operand(&mut self, frame_idx: usize) -> u16 {
        let hi = self.read_byte_operand(frame_idx) as u16;
        let lo = self.read_byte_operand(frame_idx) as u16;
        (hi << 8) | lo
    }

    #[inline]
    fn read_nombre_constante(&mut self, frame_idx: usize) -> Rc<str> {
        let idx = self.read_u16_operand(frame_idx) as usize;
        match &self.frames[frame_idx].closure.funcion.chunk.constants[idx] {
            Value::Texto(texto) => Rc::clone(texto),
            _ => panic!("constante de nombre no es texto"),
        }
    }

    fn finish_return(&mut self, resultado: Value) -> Option<Value> {
        let frame = self.frames.pop().expect("frame activo");
        self.close_upvalues(frame.base_slot);
        self.stack.truncate(frame.base_slot);
        if let Some(init) = frame.init {
            self.completar_modulo(init);
            return None;
        }
        if self.frames.is_empty() {
            Some(resultado)
        } else {
            self.push(resultado);
            None
        }
    }

    fn handle_error(&mut self, mensaje: String) -> bool {
        while let Some(frame_idx) = self.frames.len().checked_sub(1) {
            if let Some(handler) = self.frames[frame_idx].handlers.pop() {
                self.close_upvalues(handler.stack_depth);
                self.stack.truncate(handler.stack_depth);
                let slot = self.frames[frame_idx].base_slot + handler.error_slot as usize;
                self.stack[slot] = self.alloc_text(mensaje);
                self.frames[frame_idx].ip = handler.catch_ip;
                return true;
            }

            let frame = self.frames.pop().expect("frame activo");
            self.close_upvalues(frame.base_slot);
            self.stack.truncate(frame.base_slot);
            if let Some(init) = frame.init {
                self.abandonar_modulo(&init);
            }
        }
        false
    }

    fn vm_error_to_diagnostic(
        &self,
        frame_idx: usize,
        op_offset: usize,
        err: &VmError,
    ) -> WnDiagnostic {
        let frame = &self.frames[frame_idx];
        let source: Arc<SourceFile> = frame.closure.funcion.chunk.source.clone();
        let span = frame.closure.funcion.chunk.spans[op_offset].clone();
        let source_span = source.span(span.start, span.end);

        match err {
            VmError::VarNoDefinida(nombre) => {
                WnDiagnostic::var_no_definida(&source, source_span, nombre.clone())
            }
            VmError::TiposIncompatibles { .. }
            | VmError::NegacionInvalida(_)
            | VmError::TipoInvalido(_) => {
                WnDiagnostic::tipo_invalido(&source, source_span, err.to_string())
            }
            VmError::DivisionPorCero => WnDiagnostic::division_por_cero(&source, source_span),
            VmError::NoLlamable(nombre) => {
                WnDiagnostic::no_llamable(&source, source_span, nombre.clone())
            }
            VmError::NumArgInvalido {
                esperados,
                recibidos,
            } => WnDiagnostic::num_arg_invalido(&source, source_span, *esperados, *recibidos),
            VmError::IndiceInvalido { indice, largo } => {
                WnDiagnostic::indice_invalido(&source, source_span, *indice, *largo)
            }
            VmError::ClaveInexistente { clave } => {
                WnDiagnostic::clave_inexistente(&source, source_span, clave.clone())
            }
            VmError::TextoNoConvertibleANumero(valor) => {
                WnDiagnostic::texto_no_convertible(&source, source_span, valor.clone())
            }
            VmError::CampoNoExisteEnModulo { .. } | VmError::NoEsModulo(_) => {
                WnDiagnostic::runtime(&source, source_span, err.to_string())
            }
            VmError::ModuloNoEncontrado(modulo) => {
                let archivo = format!("{}.cl", modulo.replace("::", "/"));
                WnDiagnostic::modulo_no_encontrado(
                    &source,
                    source_span,
                    modulo.clone(),
                    format!(
                        "Busqué `{archivo}` desde la raíz del proyecto y entre los módulos de la biblioteca estándar (`queri std::cadena`). Revisá el nombre y la ruta."
                    ),
                )
            }
            VmError::Resolucion { modulo, error } => match error {
                ErrorResolucion::Reservado(nombre) => {
                    WnDiagnostic::modulo_reservado(&source, source_span, nombre.clone())
                }
                ErrorResolucion::FueraDeRaiz { ruta } => WnDiagnostic::modulo_ilegible(
                    &source,
                    source_span,
                    modulo.clone(),
                    format!("'{ruta}' queda fuera de la raíz del proyecto"),
                ),
                ErrorResolucion::Ilegible { ruta, causa } => WnDiagnostic::modulo_ilegible(
                    &source,
                    source_span,
                    modulo.clone(),
                    format!("{ruta}: {causa}"),
                ),
                ErrorResolucion::Conflicto { candidatos } => WnDiagnostic::modulo_conflicto(
                    &source,
                    source_span,
                    modulo.clone(),
                    format!("lo reclaman: {}", candidatos.join(" y ")),
                ),
            },
            VmError::ImportCircular(cadena) => {
                WnDiagnostic::import_circular(&source, source_span, cadena.clone())
            }
            VmError::NombreEnUso(nombre) => {
                WnDiagnostic::nombre_en_uso(&source, source_span, nombre.clone())
            }
            VmError::NoExportado { modulo, campo } => {
                WnDiagnostic::no_exportado(&source, source_span, modulo.clone(), campo.clone())
            }
            VmError::ExportInexistente {
                modulo,
                campo,
                disponibles,
            } => {
                let ayuda = if disponibles.is_empty() {
                    format!(
                        "El módulo '{modulo}' no exporta nada todavía; marcá con `convida` lo que quieras compartir."
                    )
                } else {
                    format!("Lo que '{modulo}' comparte: {}.", disponibles.join(", "))
                };
                WnDiagnostic::export_inexistente(&source, source_span, campo.clone(), ayuda)
            }
            VmError::Carga { modulo, causa } => WnDiagnostic::error_en_modulo(
                &source,
                source_span,
                self.cadena_imports(Some(modulo)),
                (**causa).clone(),
            ),
            VmError::EntradaAgotada => WnDiagnostic::runtime(&source, source_span, err.to_string()),
            VmError::OpcodeInvalido(_) | VmError::StackUnderflow => {
                WnDiagnostic::interno(err.to_string())
            }
        }
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new()
    }
}

fn resolver_indice(i: i64, len: usize) -> Result<usize, VmError> {
    let idx = if i < 0 {
        let pos = len as i64 + i;
        if pos < 0 {
            return Err(VmError::IndiceInvalido {
                indice: i,
                largo: len,
            });
        }
        pos as usize
    } else {
        i as usize
    };
    if idx >= len {
        return Err(VmError::IndiceInvalido {
            indice: i,
            largo: len,
        });
    }
    Ok(idx)
}

fn numeric_idx(idx: &Value, contexto: &str) -> Result<i64, VmError> {
    match idx {
        Value::Numero(n) => {
            const I64_MIN_F64: f64 = i64::MIN as f64;
            const I64_MAX_EXCLUSIVE_F64: f64 = -(i64::MIN as f64);
            let fuera_de_rango_i64 = *n <= I64_MIN_F64 || *n >= I64_MAX_EXCLUSIVE_F64;
            if !n.is_finite() || n.fract() != 0.0 || fuera_de_rango_i64 {
                return Err(VmError::TipoInvalido(format!(
                    "Los índices de {contexto} deben ser números enteros finitos dentro del rango soportado."
                )));
            }
            Ok(*n as i64)
        }
        other => Err(VmError::TipoInvalido(format!(
            "Los índices de {contexto} deben ser números, no '{}'.",
            other.tipo_nombre()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::compiler::compilar;
    use wn::{lexer::tokenizar, parser::parsear};
    use wn_diagnostics::SourceFile;

    fn run_src(src: &str) -> VM {
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        let source = Arc::new(SourceFile::new("<test>", src));
        let chunk = compilar(&stmts, source).unwrap();
        let mut vm = VM::new();
        vm.run(&chunk).unwrap();
        vm
    }

    fn exec_src(vm: &mut VM, src: &str) {
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        let source = Arc::new(SourceFile::new("<test>", src));
        let chunk = compilar(&stmts, source).unwrap();
        vm.run(&chunk).unwrap();
    }

    fn run_err(src: &str) -> WnDiagnostic {
        let tokens = tokenizar(src).unwrap();
        let stmts = parsear(tokens, src, "<test>").unwrap();
        let source = Arc::new(SourceFile::new("<test>", src));
        let chunk = compilar(&stmts, source).unwrap();
        let mut vm = VM::new();
        vm.run(&chunk).unwrap_err()
    }

    #[test]
    fn global_aritmetica() {
        let vm = run_src("wea x = 10 + 20");
        assert_eq!(vm.globals["x"], Value::Numero(30.0));
    }

    #[test]
    fn funciones_basicas() {
        let vm = run_src("pega sumar(a, b) { devolver a + b }\nwea x = sumar(10, 32)");
        assert_eq!(vm.globals["x"], Value::Numero(42.0));
    }

    #[test]
    fn funcion_retorna_ultima_expresion_implicita() {
        let vm = run_src("pega identidad(x) { x }\nwea r = identidad(9)");
        assert_eq!(vm.globals["r"], Value::Numero(9.0));
    }

    #[test]
    fn retorno_implicito_funciona_en_cachai() {
        let vm = run_src(
            "pega fibonacci(n) { cachai (n <= 1) { n } si no { fibonacci(n - 1) + fibonacci(n - 2) } }\nwea r = fibonacci(10)",
        );
        assert_eq!(vm.globals["r"], Value::Numero(55.0));
    }

    #[test]
    fn closures_capturan_scope_externo() {
        let vm = run_src(
            "pega afuera(x) { pega adentro() { devolver x }\n devolver adentro }\nwea f = afuera(9)\nwea r = f()",
        );
        assert_eq!(vm.globals["r"], Value::Numero(9.0));
    }

    #[test]
    fn lista_y_mapa_funcionan() {
        let vm = run_src("wea a = [10, 20, 30][1]\nwea b = {\"x\": 42}[\"x\"]");
        assert_eq!(vm.globals["a"], Value::Numero(20.0));
        assert_eq!(vm.globals["b"], Value::Numero(42.0));
    }

    #[test]
    fn para_sobre_lista_hace_snapshot() {
        let vm = run_src(
            "wea suma = 0\nwea xs = [1, 2, 3]\npara (x en xs) { suma = suma + x\n xs = [99] }\n",
        );
        assert_eq!(vm.globals["suma"], Value::Numero(6.0));
    }

    #[test]
    fn para_sobre_texto_entrega_caracteres() {
        let vm = run_src("wea r = \"\"\npara (ch en \"ola\") { r = r + ch }");
        assert_eq!(vm.globals["r"], Value::from("ola"));
    }

    #[test]
    fn ojo_captura_error_de_funcion_llamada() {
        let vm = run_src(
            "pega romper() { devolver numero(\"nope\") }\nwea r = \"ok\"\nojo { r = romper() } cago(e) { r = e }",
        );
        assert!(
            matches!(&vm.globals["r"], Value::Texto(texto) if texto.contains("No pude convertir"))
        );
    }

    #[test]
    fn cortala_y_sigue_funcionan() {
        let vm = run_src(
            "wea suma = 0\nwea i = 0\nmientras (i < 6) { i = i + 1\ncachai (i == 2) { sigue }\ncachai (i == 5) { cortala }\nsuma = suma + i }",
        );
        assert_eq!(vm.globals["suma"], Value::Numero(8.0));
    }

    #[test]
    fn gc_sweep_limpia_temporales_inaccesibles() {
        let mut vm =
            run_src("wea viva = [1]\ncachai (verdad) { wea tmp = [2, 3]\nwea otro = {\"x\": 9} }");
        let antes = vm.gc_stats();
        vm.collect_garbage();
        let despues = vm.gc_stats();
        assert!(despues.tracked_objects < antes.tracked_objects);
        assert!(matches!(vm.globals["viva"], Value::Lista(_)));
    }

    #[test]
    fn gc_conserva_closure_y_upvalue_cerrada() {
        let mut vm = VM::new();
        exec_src(
            &mut vm,
            "pega crear() { wea xs = [41]\npega leer() { devolver xs[0] }\ndevolver leer }\nwea f = crear()",
        );
        vm.collect_garbage();
        exec_src(&mut vm, "wea r = f()");
        assert_eq!(vm.globals["r"], Value::Numero(41.0));
    }

    #[test]
    fn comparacion_texto() {
        let vm = run_src("wea r = \"b\" > \"a\"");
        assert_eq!(vm.globals["r"], Value::Booleano(true));
    }

    #[test]
    fn nativa_numero() {
        let vm = run_src("wea r = numero(\"42\")");
        assert_eq!(vm.globals["r"], Value::Numero(42.0));
    }

    #[test]
    fn variable_no_definida_apunta_al_identificador() {
        let err = run_err("wea x = zeta");
        match err {
            WnDiagnostic::VarNoDefinida { span, .. } => {
                assert_eq!(span.offset(), 8usize);
                assert_eq!(span.len(), 4);
            }
            other => panic!("diagnóstico inesperado: {other:?}"),
        }
    }

    #[test]
    fn error_de_nativa_apunta_a_la_llamada() {
        let err = run_err(r#"numero("hola")"#);
        match err {
            WnDiagnostic::TextoNoConvertibleANumero { span, .. } => {
                assert_eq!(span.offset(), 0usize);
                assert_eq!(span.len(), 14);
            }
            other => panic!("diagnóstico inesperado: {other:?}"),
        }
    }

    #[test]
    fn pregunta_sin_entrada_falla_con_runtime_explicito() {
        let tokens = tokenizar(r#"pregunta("Edad: ")"#).unwrap();
        let stmts = parsear(tokens, r#"pregunta("Edad: ")"#, "<test>").unwrap();
        let source = Arc::new(SourceFile::new("<test>", r#"pregunta("Edad: ")"#));
        let chunk = compilar(&stmts, source).unwrap();
        let mut vm = VM::con_io(Vec::<u8>::new(), Cursor::new(Vec::<u8>::new()));

        let err = vm.run(&chunk).unwrap_err();

        match err {
            WnDiagnostic::Runtime { mensaje, .. } => {
                assert_eq!(mensaje, "pregunta() pidió más entrada que la provista.");
            }
            other => panic!("diagnóstico inesperado: {other:?}"),
        }
    }
}
