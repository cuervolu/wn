//! Puente transitorio hacia la raíz única de diagnósticos.
//!
//! La implementación real vive en `wn-diagnostics`. Este módulo existe solo
//! para no romper de golpe todo el árbol mientras desaparece el intérprete
//! tree-walker.

pub use wn_diagnostics::WnDiagnostic as WnError;
pub use wn_diagnostics::{SourceFile, WnDiagnostic};
