//! Tipos de valor que el VM puede manipular.
//!
//! Por ahora los objetos heap viven con `Rc`/`RefCell`. Eso permite cerrar la
//! semántica del VM antes de acoplar el recolector final. La capa de GC se monta
//! encima de estas referencias rastreando qué objetos siguen alcanzables.

use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::chunk::Chunk;
pub use crate::native::NativeFn;

/// Descriptor compile-time de una captura léxica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpvalueDescriptor {
    pub index: u8,
    pub is_local: bool,
}

/// Función compilada a bytecode.
#[derive(Debug)]
pub struct ObjFunction {
    pub chunk: Chunk,
    pub aridad: usize,
    pub nombre: String,
    pub upvalues: Vec<UpvalueDescriptor>,
}

/// Una variable capturada por una closure.
#[derive(Debug, Clone)]
pub enum UpvalueState {
    /// La variable sigue viviendo en el stack del frame creador.
    Abierta(usize),
    /// El frame creador ya murió; el valor quedó cerrado en heap.
    Cerrada(Value),
}

/// Upvalue compartida entre una o más closures.
#[derive(Debug)]
pub struct ObjUpvalue {
    pub state: UpvalueState,
}

/// Closure runtime: función compilada + valores capturados.
#[derive(Debug)]
pub struct ObjClosure {
    pub funcion: Rc<ObjFunction>,
    pub upvalues: RefCell<Vec<Rc<RefCell<ObjUpvalue>>>>,
}

/// Iterador interno del VM. Siempre snapshot-ea al iniciar el `para`.
#[derive(Debug, Clone)]
pub enum ObjIterator {
    Lista { items: Vec<Value>, index: usize },
    Texto { chars: Vec<String>, index: usize },
}

/// Un valor en el stack del VM o en el pool de constantes.
#[derive(Debug, Clone)]
pub enum Value {
    Numero(f64),
    Booleano(bool),
    Nada,
    Texto(Rc<str>),
    Lista(Rc<RefCell<Vec<Value>>>),
    Mapa(Rc<RefCell<HashMap<String, Value>>>),
    Funcion(Rc<ObjFunction>),
    Closure(Rc<ObjClosure>),
    Nativa(NativeFn),
    Iterador(Rc<RefCell<ObjIterator>>),
}

impl Value {
    /// Mantiene paridad con el runtime viejo:
    /// `0`, `falso`, `nada` y texto vacío son falsy.
    pub fn es_verdadero(&self) -> bool {
        match self {
            Value::Numero(n) => *n != 0.0,
            Value::Booleano(b) => *b,
            Value::Nada => false,
            Value::Texto(s) => !s.is_empty(),
            Value::Lista(_)
            | Value::Mapa(_)
            | Value::Funcion(_)
            | Value::Closure(_)
            | Value::Nativa(_)
            | Value::Iterador(_) => true,
        }
    }

    /// Nombre del tipo para mensajes de error con sabor chileno.
    pub fn tipo_nombre(&self) -> &'static str {
        match self {
            Value::Numero(_) => "numero",
            Value::Booleano(_) => "booleano",
            Value::Nada => "nada",
            Value::Texto(_) => "texto",
            Value::Lista(_) => "lista",
            Value::Mapa(_) => "mapa",
            Value::Funcion(_) | Value::Closure(_) | Value::Nativa(_) => "pega",
            Value::Iterador(_) => "iterador",
        }
    }

    /// Convierte cualquier valor a clave de mapa usando la misma regla que el
    /// intérprete antiguo: los textos se preservan, lo demás usa `Display`.
    pub fn a_clave_mapa(&self) -> String {
        match self {
            Value::Texto(s) => s.to_string(),
            other => other.to_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Numero(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Booleano(b) => write!(f, "{}", if *b { "verdad" } else { "falso" }),
            Value::Nada => write!(f, "nada"),
            Value::Texto(s) => write!(f, "{s}"),
            Value::Lista(items) => {
                let items = items.borrow();
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Mapa(map) => {
                let map = map.borrow();
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k:?}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Funcion(func) => write!(f, "<pega {}>", func.nombre),
            Value::Closure(closure) => write!(f, "<pega {}>", closure.funcion.nombre),
            Value::Nativa(_) => write!(f, "<pega nativa>"),
            Value::Iterador(_) => write!(f, "<iterador>"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Numero(a), Value::Numero(b)) => a == b,
            (Value::Booleano(a), Value::Booleano(b)) => a == b,
            (Value::Nada, Value::Nada) => true,
            (Value::Texto(a), Value::Texto(b)) => a == b,
            _ => false,
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Numero(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Booleano(b)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Texto(Rc::from(s))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Texto(Rc::from(s.as_str()))
    }
}
