//! Implementación de [`ModuleResolver`] para la biblioteca estándar de WN++.
//!
//! Este archivo va en `crates/wn-stdlib/src/stdlib_resolver.rs`.
//! `wn-cli` lo usa para conectar la stdlib al VM.
//!
//! ```rust
//! // En wn-cli/src/main.rs
//! use wn_stdlib::stdlib_resolver::StdlibResolver;
//! use wn_stdlib::stdlib_resolver::CompositeResolver;
//!
//! let resolver = Box::new(CompositeResolver::new(vec![
//!     Box::new(StdlibResolver),
//! ]));
//! let vm = VM::con_resolver(io::stdout(), BufReader::new(io::stdin()), resolver);
//! ```

use crate::lista::LISTA;
use crate::cadena::CADENA;
use std::{collections::HashMap, rc::Rc};
use wn_vm::{native::NativeFn, resolver::ModuleResolver, value::Value};

/// Resolver de módulos de la biblioteca estándar.
///
/// Mapeo de paths a módulos:
/// ```text
/// "cadena"      → módulo cadena
/// "std::cadena" → módulo cadena  (alias con namespace explícito)
/// "lista"      → módulo lista
/// "std::lista" → módulo lista
/// ```
pub struct StdlibResolver;

impl ModuleResolver for StdlibResolver {
    fn resolver(&self, path: &[&str]) -> Option<Value> {
        match path {
            ["cadena"] | ["std", "cadena"] => Some(construir_modulo(CADENA)),
            ["lista"] | ["std", "lista"] => Some(construir_modulo(LISTA)),
            _ => None,
        }
    }
}

/// Convierte un slice de [`NativeFn`] en un [`Value::Modulo`].
fn construir_modulo(funciones: &[NativeFn]) -> Value {
    let map: HashMap<String, Value> = funciones
        .iter()
        .map(|f| (f.nombre.to_string(), Value::Nativa(*f)))
        .collect();
    Value::Modulo(Rc::new(map))
}
