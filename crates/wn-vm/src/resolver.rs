//! Interfaz de resolución de módulos para el VM.
//!
//! El VM no sabe de dónde vienen los módulos. Solo llama a [`ModuleResolver`].
//! `wn-cli` conecta las implementaciones concretas (`StdlibResolver`, `FileResolver`)
//! al arrancar.
//!
//! Resolver solo responde "¿qué es este módulo?". Compilarlo, instanciarlo y
//! ejecutarlo es trabajo del VM (así hay un único compilador y una única VM).

use std::sync::Arc;

use wn_diagnostics::SourceFile;

use crate::value::Value;

/// Resultado de resolver un path de módulo.
#[derive(Debug, Clone)]
pub enum ModuloResuelto {
    /// Módulo nativo ya construido (biblioteca estándar).
    Nativo(Value),
    /// Código fuente `.cl` de un módulo de usuario.
    ///
    /// `id` es la identidad normalizada del módulo (ruta canónica): dos imports
    /// que apunten al mismo archivo comparten `id` y, por lo tanto, la misma instancia.
    Fuente { id: String, source: Arc<SourceFile> },
}

/// Por qué un resolver no pudo resolver un módulo que sí parecía suyo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorResolucion {
    /// El primer segmento (`std`) está reservado para la biblioteca estándar.
    Reservado(String),
    /// El path sale de la raíz del proyecto (p. ej. por un symlink).
    FueraDeRaiz { ruta: String },
    /// El archivo existe pero no se pudo leer.
    Ilegible { ruta: String, causa: String },
    /// Más de un resolver reclama el mismo path.
    Conflicto { candidatos: Vec<String> },
}

/// Resuelve un path de módulo.
///
/// El path llega como slice de strings, ej: `["std", "cadena"]` o `["utilidades", "texto"]`.
///
/// # Contrato
/// - `Ok(Some(_))`: el módulo existe.
/// - `Ok(None)`: este resolver no conoce ese módulo (otro podría conocerlo).
/// - `Err(_)`: el módulo parece suyo pero no se puede usar.
pub trait ModuleResolver {
    fn resolver(&self, path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion>;
}

/// Resolver por defecto: nunca encuentra nada.
///
/// Usado por `VM::new()` para no romper tests existentes.
/// `wn-cli` reemplaza esto con el resolver real.
pub struct NoopResolver;

impl ModuleResolver for NoopResolver {
    fn resolver(&self, _path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion> {
        Ok(None)
    }
}

/// Combina múltiples resolvers.
///
/// Consulta a todos. Si exactamente uno responde, se usa su resultado. Si más de uno
/// responde (p. ej. `cadena` en la stdlib y un `cadena.cl` local) es un
/// [`ErrorResolucion::Conflicto`]: nada se oculta en silencio. Los errores de un
/// resolver solo se muestran si ningún otro encontró el módulo.
///
/// ```ignore
/// let resolver = CompositeResolver::new(vec![
///     Box::new(StdlibResolver),
///     Box::new(FileResolver::new(raiz)?),
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
    fn resolver(&self, path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion> {
        let mut encontrados = Vec::new();
        let mut primer_error = None;
        for resolver in &self.resolvers {
            match resolver.resolver(path) {
                Ok(Some(modulo)) => encontrados.push(modulo),
                Ok(None) => {}
                Err(err) => {
                    primer_error.get_or_insert(err);
                }
            }
        }
        match encontrados.len() {
            0 => primer_error.map_or(Ok(None), Err),
            1 => Ok(encontrados.pop()),
            _ => Err(ErrorResolucion::Conflicto {
                candidatos: encontrados
                    .iter()
                    .map(|modulo| match modulo {
                        ModuloResuelto::Nativo(_) => "la biblioteca estándar".to_string(),
                        ModuloResuelto::Fuente { id, .. } => id.clone(),
                    })
                    .collect(),
            }),
        }
    }
}
