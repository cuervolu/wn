//! Interfaz de resolución de módulos para el VM.
//!
//! El VM no sabe de dónde vienen los módulos. Solo llama a [`ModuleResolver`].
//! `wn-cli` conecta la implementación concreta (`StdlibResolver`, `FileResolver`)
//! al arrancar.

use crate::value::Value;

/// Resuelve un path de módulo a un [`Value::Modulo`].
///
/// El path llega como slice de strings, ej: `["std", "texto"]` o `["utils"]`.
///
/// # Contrato
/// Si el módulo existe, retorna `Some(Value::Modulo(...))`.
/// Si no existe, retorna `None`, el VM convierte eso en error de runtime.
pub trait ModuleResolver {
    fn resolver(&self, path: &[&str]) -> Option<Value>;
}

/// Resolver por defecto: nunca encuentra nada.
///
/// Usado por `VM::new()` para no romper tests existentes.
/// `wn-cli` reemplaza esto con el resolver real.
pub struct NoopResolver;

impl ModuleResolver for NoopResolver {
    fn resolver(&self, _path: &[&str]) -> Option<Value> {
        None
    }
}

/// Combina múltiples resolvers en orden: retorna el primero que encuentre el módulo.
///
/// ```
/// let resolver = CompositeResolver::new(vec![
///     Box::new(StdlibResolver::new()),
///     Box::new(FileResolver::new(paths)),
/// ]);
/// ```
pub struct CompositeResolver {
    resolvers: Vec<Box<dyn ModuleResolver>>,
}

impl CompositeResolver {
    pub fn new(resolvers: Vec<Box<dyn ModuleResolver>>) -> Self {
        Self { resolvers }
    }
}

impl ModuleResolver for CompositeResolver {
    fn resolver(&self, path: &[&str]) -> Option<Value> {
        self.resolvers.iter().find_map(|r| r.resolver(path))
    }
}
