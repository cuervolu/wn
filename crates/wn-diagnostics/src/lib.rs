use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

/// Archivo fuente reusable para fabricar diagnósticos con `miette`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    filename: String,
    src: String,
}

impl SourceFile {
    pub fn new(filename: impl Into<String>, src: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            src: src.into(),
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn named_source(&self) -> NamedSource<String> {
        NamedSource::new(&self.filename, self.src.clone())
    }

    pub fn span(&self, start: usize, end: usize) -> SourceSpan {
        let len = end.saturating_sub(start);
        SourceSpan::new(start.into(), len)
    }

    pub fn eof_span(&self) -> SourceSpan {
        let offset = self.src.len();
        SourceSpan::new(offset.into(), 0usize)
    }

    pub fn line_for_offset(&self, offset: usize) -> u32 {
        let mut line = 1u32;
        for ch in self.src[..offset.min(self.src.len())].chars() {
            if ch == '\n' {
                line += 1;
            }
        }
        line
    }
}

#[derive(Debug, Error, Diagnostic, Clone)]
pub enum WnDiagnostic {
    #[error("Error léxico")]
    #[diagnostic(code(wn::lexico), help("Revisa el carácter problemático"))]
    Lexico {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("Error de sintaxis")]
    #[diagnostic(code(wn::sintaxis))]
    Sintaxis {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("Error de compilación")]
    #[diagnostic(code(wn::compilacion))]
    Compilacion {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("La wea '{nombre}' no existe papito.")]
    #[diagnostic(
        code(wn::runtime::var_no_definida),
        help("¿Escribiste bien el nombre? Las variables se declaran con `wea {nombre} = valor`.")
    )]
    VarNoDefinida {
        #[source_code]
        src: NamedSource<String>,
        #[label("variable no definida")]
        span: SourceSpan,
        nombre: String,
    },

    #[error("Oe, '{nombre}' es duro, no lo podí cambiar.")]
    #[diagnostic(
        code(wn::runtime::constante_inmutable),
        help("Si necesitái cambiar el valor, declarálo con `wea` en vez de `duro`.")
    )]
    ConstanteInmutable {
        #[source_code]
        src: NamedSource<String>,
        #[label("constante inmutable")]
        span: SourceSpan,
        nombre: String,
    },

    #[error("Weon, no se puede dividir por cero.")]
    #[diagnostic(
        code(wn::runtime::division_por_cero),
        help(
            "Revisá que el divisor no pueda ser cero. Podés usar `cachai (divisor != 0)` antes de dividir."
        )
    )]
    DivisionPorCero {
        #[source_code]
        src: NamedSource<String>,
        #[label("división por cero")]
        span: SourceSpan,
    },

    #[error("Te fuiste al chancho, el índice {indice} no existe en la lista (largo: {largo}).")]
    #[diagnostic(
        code(wn::runtime::indice_invalido),
        help("Los índices parten de 0. El último elemento está en la posición {largo} - 1.")
    )]
    IndiceInvalido {
        #[source_code]
        src: NamedSource<String>,
        #[label("índice inválido")]
        span: SourceSpan,
        indice: i64,
        largo: usize,
    },

    #[error("La clave '{clave}' no existe en el mapa papito.")]
    #[diagnostic(
        code(wn::runtime::clave_inexistente),
        help("Revisá las claves del mapa antes de acceder. Podés usar `cachar()` para debuggear.")
    )]
    ClaveInexistente {
        #[source_code]
        src: NamedSource<String>,
        #[label("clave inexistente")]
        span: SourceSpan,
        clave: String,
    },

    #[error("'{nombre}' no es una pega papito.")]
    #[diagnostic(
        code(wn::runtime::no_llamable),
        help(
            "Solo podís llamar pegas declaradas con `pega`. Usá `cachar({nombre})` para ver su tipo."
        )
    )]
    NoLlamable {
        #[source_code]
        src: NamedSource<String>,
        #[label("valor no llamable")]
        span: SourceSpan,
        nombre: String,
    },

    #[error("La pega espera {esperados} argumento(s), le pasaste {recibidos}.")]
    #[diagnostic(
        code(wn::runtime::num_arg_invalido),
        help("Revisá la firma de la pega y ajustá los argumentos que le estái pasando.")
    )]
    NumArgInvalido {
        #[source_code]
        src: NamedSource<String>,
        #[label("aridad inválida")]
        span: SourceSpan,
        esperados: usize,
        recibidos: usize,
    },

    #[error("Error de tipos: {mensaje}")]
    #[diagnostic(
        code(wn::runtime::tipo_invalido),
        help("Usá `cachar(valor)` para ver el tipo de una variable antes de operar con ella.")
    )]
    TipoInvalido {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("No pude convertir {valor:?} a número.")]
    #[diagnostic(
        code(wn::runtime::texto_no_convertible_a_numero),
        help("A `numero()` pásale un texto numérico simple como `42`, `-7` o `3.14`.")
    )]
    TextoNoConvertibleANumero {
        #[source_code]
        src: NamedSource<String>,
        #[label("texto no convertible")]
        span: SourceSpan,
        valor: String,
    },

    #[error("Error en tiempo de ejecución: {mensaje}")]
    #[diagnostic(code(wn::runtime), help("Revisa la lógica de tu programa ctm."))]
    Runtime {
        #[source_code]
        src: NamedSource<String>,
        #[label("{mensaje}")]
        span: SourceSpan,
        mensaje: String,
    },

    #[error("Bug interno del motor: {mensaje}")]
    #[diagnostic(
        code(wn::interno),
        help("Esto es un bug del motor. Reportalo con el programa que lo dispara.")
    )]
    Interno { mensaje: String },
}

impl WnDiagnostic {
    pub fn lexico(source: &SourceFile, span: SourceSpan, mensaje: impl Into<String>) -> Self {
        Self::Lexico {
            src: source.named_source(),
            span,
            mensaje: mensaje.into(),
        }
    }

    pub fn sintaxis(source: &SourceFile, span: SourceSpan, mensaje: impl Into<String>) -> Self {
        Self::Sintaxis {
            src: source.named_source(),
            span,
            mensaje: mensaje.into(),
        }
    }

    pub fn compilacion(source: &SourceFile, span: SourceSpan, mensaje: impl Into<String>) -> Self {
        Self::Compilacion {
            src: source.named_source(),
            span,
            mensaje: mensaje.into(),
        }
    }

    pub fn var_no_definida(
        source: &SourceFile,
        span: SourceSpan,
        nombre: impl Into<String>,
    ) -> Self {
        Self::VarNoDefinida {
            src: source.named_source(),
            span,
            nombre: nombre.into(),
        }
    }

    pub fn constante_inmutable(
        source: &SourceFile,
        span: SourceSpan,
        nombre: impl Into<String>,
    ) -> Self {
        Self::ConstanteInmutable {
            src: source.named_source(),
            span,
            nombre: nombre.into(),
        }
    }

    pub fn division_por_cero(source: &SourceFile, span: SourceSpan) -> Self {
        Self::DivisionPorCero {
            src: source.named_source(),
            span,
        }
    }

    pub fn indice_invalido(
        source: &SourceFile,
        span: SourceSpan,
        indice: i64,
        largo: usize,
    ) -> Self {
        Self::IndiceInvalido {
            src: source.named_source(),
            span,
            indice,
            largo,
        }
    }

    pub fn clave_inexistente(
        source: &SourceFile,
        span: SourceSpan,
        clave: impl Into<String>,
    ) -> Self {
        Self::ClaveInexistente {
            src: source.named_source(),
            span,
            clave: clave.into(),
        }
    }

    pub fn no_llamable(source: &SourceFile, span: SourceSpan, nombre: impl Into<String>) -> Self {
        Self::NoLlamable {
            src: source.named_source(),
            span,
            nombre: nombre.into(),
        }
    }

    pub fn num_arg_invalido(
        source: &SourceFile,
        span: SourceSpan,
        esperados: usize,
        recibidos: usize,
    ) -> Self {
        Self::NumArgInvalido {
            src: source.named_source(),
            span,
            esperados,
            recibidos,
        }
    }

    pub fn tipo_invalido(
        source: &SourceFile,
        span: SourceSpan,
        mensaje: impl Into<String>,
    ) -> Self {
        Self::TipoInvalido {
            src: source.named_source(),
            span,
            mensaje: mensaje.into(),
        }
    }

    pub fn texto_no_convertible(
        source: &SourceFile,
        span: SourceSpan,
        valor: impl Into<String>,
    ) -> Self {
        Self::TextoNoConvertibleANumero {
            src: source.named_source(),
            span,
            valor: valor.into(),
        }
    }

    pub fn runtime(source: &SourceFile, span: SourceSpan, mensaje: impl Into<String>) -> Self {
        Self::Runtime {
            src: source.named_source(),
            span,
            mensaje: mensaje.into(),
        }
    }

    pub fn interno(mensaje: impl Into<String>) -> Self {
        Self::Interno {
            mensaje: mensaje.into(),
        }
    }

    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Lexico { span, .. }
            | Self::Sintaxis { span, .. }
            | Self::Compilacion { span, .. }
            | Self::VarNoDefinida { span, .. }
            | Self::ConstanteInmutable { span, .. }
            | Self::DivisionPorCero { span, .. }
            | Self::IndiceInvalido { span, .. }
            | Self::ClaveInexistente { span, .. }
            | Self::NoLlamable { span, .. }
            | Self::NumArgInvalido { span, .. }
            | Self::TipoInvalido { span, .. }
            | Self::TextoNoConvertibleANumero { span, .. }
            | Self::Runtime { span, .. } => Some(*span),
            Self::Interno { .. } => None,
        }
    }
}
