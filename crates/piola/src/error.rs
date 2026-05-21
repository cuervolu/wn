use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum PiolaError {
    #[error("Error léxico")]
    #[diagnostic(code(piola::lexico), help("Revisa el carácter problemático"))]
    Lexico {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("Error de sintaxis")]
    #[diagnostic(code(piola::sintaxis))]
    Sintaxis {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("Error en tiempo de ejecución: {mensaje}")]
    #[diagnostic(code(piola::runtime), help("Revisa la lógica de tu programa ctm"))]
    Runtime { mensaje: String },
}
