//! Implementación de [`ModuleResolver`] para la biblioteca estándar de WN++.
//!
//! `wn-cli` lo usa para conectar la stdlib al VM, junto al resolver de archivos:
//!
//! ```ignore
//! let resolver = Box::new(CompositeResolver::new(vec![
//!     Box::new(StdlibResolver),
//!     Box::new(FileResolver::new(raiz)?),
//! ]));
//! let vm = VM::con_resolver(io::stdout(), BufReader::new(io::stdin()), resolver);
//! ```

use crate::cadena::CADENA;
use crate::lista::LISTA;
use std::{collections::HashMap, rc::Rc};
use wn_vm::{
    native::NativeFn,
    resolver::{ErrorResolucion, ModuleResolver, ModuloResuelto},
    value::Value,
};

/// Resolver de módulos de la biblioteca estándar.
///
/// Mapeo de paths a módulos:
/// ```text
/// "cadena"       módulo cadena
/// "std::cadena"  módulo cadena  (alias con namespace explícito)
/// "lista"       módulo lista
/// "std::lista"  módulo lista
/// ```
pub struct StdlibResolver;

impl ModuleResolver for StdlibResolver {
    fn resolver(&self, path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion> {
        let modulo = match path {
            ["cadena"] | ["std", "cadena"] => construir_modulo(CADENA),
            ["lista"] | ["std", "lista"] => construir_modulo(LISTA),
            _ => return Ok(None),
        };
        Ok(Some(ModuloResuelto::Nativo(modulo)))
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
