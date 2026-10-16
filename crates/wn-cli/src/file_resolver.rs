//! Resolver de módulos de usuario sobre el sistema de archivos.
//!
//! Reglas (no dependen del directorio de trabajo):
//! - Todo `queri` se resuelve desde la **raíz del proyecto**, sin importar qué archivo
//!   lo escribe: `queri utilidades::texto` es `<raíz>/utilidades/texto.cl`.
//! - La raíz es el directorio del archivo de entrada (en el REPL, el directorio
//!   desde el que se lanzó). Se canonicaliza una sola vez al crear el resolver.
//! - La identidad de un módulo es su ruta canónica. dos imports equivalentes
//!   comparten instancia.
//! - Los segmentos no pueden contener separadores ni `..`, y el archivo canónico
//!   (tras seguir symlinks) debe seguir dentro de la raíz.
//! - `std` está reservado para la biblioteca estándar y nunca se busca en disco.

use std::{
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use wn_diagnostics::SourceFile;
use wn_vm::resolver::{ErrorResolucion, ModuleResolver, ModuloResuelto};

pub struct FileResolver {
    raiz: PathBuf,
}

impl FileResolver {
    /// Crea un resolver anclado a `raiz` (se canonicaliza aquí, una sola vez).
    pub fn new(raiz: &Path) -> std::io::Result<Self> {
        Ok(Self {
            raiz: raiz.canonicalize()?,
        })
    }
}

/// Un segmento válido es un único componente normal: sin `/`, `\`, `..`, `.` ni vacío.
fn segmento_valido(segmento: &str) -> bool {
    !segmento.contains(['/', '\\', '\0'])
        && matches!(
            Path::new(segmento)
                .components()
                .collect::<Vec<_>>()
                .as_slice(),
            [Component::Normal(_)]
        )
}

impl ModuleResolver for FileResolver {
    fn resolver(&self, path: &[&str]) -> Result<Option<ModuloResuelto>, ErrorResolucion> {
        let Some((ultimo, intermedios)) = path.split_last() else {
            return Ok(None);
        };
        if path[0] == "std" {
            return Err(ErrorResolucion::Reservado("std".to_string()));
        }
        if !path.iter().all(|segmento| segmento_valido(segmento)) {
            return Ok(None);
        }

        let mut candidato = self.raiz.clone();
        candidato.extend(intermedios);
        candidato.push(format!("{ultimo}.cl"));

        let canonico = match candidato.canonicalize() {
            Ok(ruta) => ruta,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            // Un directorio intermedio que es un archivo, p. ej. `a.cl/b.cl`.
            Err(e) if e.kind() == ErrorKind::NotADirectory => return Ok(None),
            Err(e) => {
                return Err(ErrorResolucion::Ilegible {
                    ruta: candidato.display().to_string(),
                    causa: e.to_string(),
                });
            }
        };
        if !canonico.starts_with(&self.raiz) {
            return Err(ErrorResolucion::FueraDeRaiz {
                ruta: canonico.display().to_string(),
            });
        }
        if !canonico.is_file() {
            return Ok(None);
        }

        let relativa = canonico.strip_prefix(&self.raiz).unwrap_or(&canonico);
        let src = std::fs::read_to_string(&canonico).map_err(|e| ErrorResolucion::Ilegible {
            ruta: relativa.display().to_string(),
            causa: e.to_string(),
        })?;
        Ok(Some(ModuloResuelto::Fuente {
            id: canonico.to_string_lossy().into_owned(),
            source: Arc::new(SourceFile::new(relativa.to_string_lossy(), src)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn proyecto(nombre: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("wn-fileresolver-{nombre}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("utilidades")).unwrap();
        fs::write(dir.join("matematicas.cl"), "convida wea x = 1").unwrap();
        fs::write(dir.join("utilidades/texto.cl"), "convida wea y = 2").unwrap();
        dir
    }

    fn id_de(r: Result<Option<ModuloResuelto>, ErrorResolucion>) -> Option<String> {
        match r.unwrap() {
            Some(ModuloResuelto::Fuente { id, .. }) => Some(id),
            _ => None,
        }
    }

    #[test]
    fn resuelve_archivo_y_subdirectorio() {
        let dir = proyecto("basico");
        let r = FileResolver::new(&dir).unwrap();
        assert!(
            id_de(r.resolver(&["matematicas"]))
                .unwrap()
                .ends_with("matematicas.cl")
        );
        assert!(
            id_de(r.resolver(&["utilidades", "texto"]))
                .unwrap()
                .ends_with("texto.cl")
        );
        assert!(r.resolver(&["no_existe"]).unwrap().is_none());
        assert!(
            r.resolver(&["utilidades"]).unwrap().is_none(),
            "un directorio no es módulo"
        );
    }

    #[test]
    fn rutas_equivalentes_tienen_la_misma_identidad() {
        let dir = proyecto("identidad");
        let a = FileResolver::new(&dir).unwrap();
        let b = FileResolver::new(&dir.join("utilidades").join("..")).unwrap();
        assert_eq!(
            id_de(a.resolver(&["matematicas"])),
            id_de(b.resolver(&["matematicas"]))
        );
    }

    #[test]
    fn std_esta_reservado() {
        let dir = proyecto("std");
        fs::write(dir.join("std.cl"), "").unwrap();
        let r = FileResolver::new(&dir).unwrap();
        assert_eq!(
            r.resolver(&["std"]).unwrap_err(),
            ErrorResolucion::Reservado("std".into())
        );
        assert!(r.resolver(&["std", "cadena"]).is_err());
    }

    #[test]
    fn rechaza_segmentos_que_escapan() {
        let dir = proyecto("escape");
        let r = FileResolver::new(&dir).unwrap();
        assert!(r.resolver(&[".."]).unwrap().is_none());
        assert!(r.resolver(&["..", "matematicas"]).unwrap().is_none());
        assert!(r.resolver(&["a/b"]).unwrap().is_none());
        assert!(r.resolver(&["/etc/passwd"]).unwrap().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn rechaza_symlink_que_sale_de_la_raiz() {
        let dir = proyecto("symlink");
        let fuera = proyecto("symlink-fuera");
        std::os::unix::fs::symlink(fuera.join("matematicas.cl"), dir.join("fuga.cl")).unwrap();
        let r = FileResolver::new(&dir).unwrap();
        assert!(matches!(
            r.resolver(&["fuga"]),
            Err(ErrorResolucion::FueraDeRaiz { .. })
        ));
    }
}
