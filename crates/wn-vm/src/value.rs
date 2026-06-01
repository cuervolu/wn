//! Tipos de valor que el VM puede manipular.
//!
//! # Diseño en dos capas
//!
//! **Inmediatos** (`Número`, `Booleano`, `Nada`): copiados por valor en el stack,
//! sin allocación, sin GC.
//!
//! **Objetos** (`Texto`, y próximamente `Lista`, `Mapa`, `Funcion`): reference-counted
//! por ahora. Cuando el GC esté listo, serán `GcRef<ObjTexto>` etc.

use std::{fmt, rc::Rc};

/// Un valor en el stack del VM o en el pool de constantes.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Numero(f64),
    Booleano(bool),
    Nada,
    /// `Rc<str>` evita clonar el string completo al duplicar valores en el stack.
    /// TODO: Reemplazar por `GcRef<ObjTexto>` cuando llegue el GC.
    Texto(Rc<str>),
}

impl Value {
    /// Solo `falso` y `nada` son falsy. Todo lo demás es truthy.
    pub fn es_verdadero(&self) -> bool {
        match self {
            Value::Nada => false,
            Value::Booleano(b) => *b,
            _ => true,
        }
    }

    /// Nombre del tipo para mensajes de error con sabor chileno.
    pub fn tipo_nombre(&self) -> &'static str {
        match self {
            Value::Numero(_) => "numero",
            Value::Booleano(_) => "booleano",
            Value::Nada => "nada",
            Value::Texto(_) => "texto",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Numero(n) => {
                // "10" en vez de "10.0" cuando el número es entero
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Booleano(b) => write!(f, "{}", if *b { "verdad" } else { "falso" }),
            Value::Nada => write!(f, "nada"),
            Value::Texto(s) => write!(f, "{s}"),
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
