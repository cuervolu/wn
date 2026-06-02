use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::ast::Stmt;

use super::env::Entorno;

#[derive(Debug, Clone, Copy)]
pub enum Nativa {
    Lorea,
    Largo,
    Cachar,
    Pregunta,
    Numero,
    Texto,
}

#[derive(Debug, Clone)]
pub enum Valor {
    Numero(f64),
    Texto(String),
    Booleano(bool),
    Nada,
    Lista(Rc<RefCell<Vec<Valor>>>),
    Mapa(Rc<RefCell<HashMap<String, Valor>>>),
    Funcion {
        params: Vec<String>,
        cuerpo: Vec<Stmt>,
        entorno: Rc<RefCell<Entorno>>,
    },
    Nativa(Nativa),
}

impl Valor {
    pub fn tipo_nombre(&self) -> &'static str {
        match self {
            Valor::Numero(_) => "numero",
            Valor::Texto(_) => "texto",
            Valor::Booleano(_) => "booleano",
            Valor::Nada => "nada",
            Valor::Lista(_) => "lista",
            Valor::Mapa(_) => "mapa",
            Valor::Funcion { .. } | Valor::Nativa(_) => "pega",
        }
    }

    pub fn es_verdadero(&self) -> bool {
        match self {
            Valor::Booleano(b) => *b,
            Valor::Nada => false,
            Valor::Numero(n) => *n != 0.0,
            Valor::Texto(s) => !s.is_empty(),
            _ => true,
        }
    }
}

impl fmt::Display for Valor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Valor::Numero(n) => {
                if n.fract() == 0.0 && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Valor::Texto(s) => write!(f, "{s}"),
            Valor::Booleano(b) => write!(f, "{}", if *b { "verdad" } else { "falso" }),
            Valor::Nada => write!(f, "nada"),
            Valor::Lista(items) => {
                let items = items.borrow();
                write!(f, "[")?;
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Valor::Mapa(map) => {
                let map = map.borrow();
                write!(f, "{{")?;
                let mut first = true;
                for (k, v) in map.iter() {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k:?}: {v}")?;
                    first = false;
                }
                write!(f, "}}")
            }
            Valor::Funcion { params, .. } => write!(f, "<pega({})>", params.join(", ")),
            Valor::Nativa(_) => write!(f, "<pega nativa>"),
        }
    }
}

impl PartialEq for Valor {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Valor::Numero(a), Valor::Numero(b)) => a == b,
            (Valor::Texto(a), Valor::Texto(b)) => a == b,
            (Valor::Booleano(a), Valor::Booleano(b)) => a == b,
            (Valor::Nada, Valor::Nada) => true,
            _ => false,
        }
    }
}
