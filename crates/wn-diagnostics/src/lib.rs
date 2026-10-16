use std::sync::Arc;

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

/// Diagnóstico original de un módulo que falló, anidado dentro de `ErrorEnModulo`.
/// Delega todo en el diagnóstico interno; existe para que miette lo pueda usar como
/// `diagnostic_source` sin perder `Clone`.
#[derive(Debug, Clone)]
pub struct CausaDiagnostico(pub Box<WnDiagnostic>);

impl std::fmt::Display for CausaDiagnostico {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CausaDiagnostico {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl Diagnostic for CausaDiagnostico {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.code()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.0.help()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.0.source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.0.labels()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.0.diagnostic_source()
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

    #[error("El módulo '{modulo}' no existe!.")]
    #[diagnostic(code(wn::modulo::no_encontrado), help("{ayuda}"))]
    ModuloNoEncontrado {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("este módulo no se encontró")]
        span: SourceSpan,
        modulo: String,
        ayuda: String,
    },

    #[error("No pude leer el módulo '{modulo}'.")]
    #[diagnostic(
        code(wn::modulo::ilegible),
        help(
            "Revisá que el archivo exista, que esté dentro de la raíz del proyecto y que tengái permiso para leerlo."
        )
    )]
    ModuloIlegible {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("{causa}")]
        span: SourceSpan,
        modulo: String,
        causa: String,
    },

    #[error("Importación circular: {cadena}")]
    #[diagnostic(
        code(wn::modulo::circular),
        help(
            "Un módulo no puede depender de sí mismo, ni directa ni indirectamente. Mueve lo compartido a un tercer módulo."
        )
    )]
    ImportCircular {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("este import cierra el ciclo")]
        span: SourceSpan,
        cadena: String,
    },

    #[error("El módulo '{modulo}' es ambiguo.")]
    #[diagnostic(
        code(wn::modulo::conflicto),
        help(
            "Renombrá el archivo local, o usá `std::{modulo}` si querís el de la biblioteca estándar."
        )
    )]
    ModuloConflicto {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("{candidatos}")]
        span: SourceSpan,
        modulo: String,
        candidatos: String,
    },

    #[error("'{nombre}' está reservado, no lo podí usar como módulo propio, copión.")]
    #[diagnostic(
        code(wn::modulo::reservado),
        help(
            "`std::` es exclusivo de la biblioteca estándar. Usá otro nombre para tu archivo o directorio."
        )
    )]
    ModuloReservado {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("nombre reservado")]
        span: SourceSpan,
        nombre: String,
    },

    #[error("'{nombre}' no es público en el módulo '{modulo}'.")]
    #[diagnostic(
        code(wn::modulo::no_exportado),
        help(
            "Para usarlo desde afuera, declaralo con `convida` en {modulo}: `convida pega {nombre}(...)` o `convida wea {nombre} = ...`."
        )
    )]
    NoExportado {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("declaración privada")]
        span: SourceSpan,
        modulo: String,
        nombre: String,
    },

    #[error("No existe '{nombre}' en el módulo importado.")]
    #[diagnostic(code(wn::modulo::export_inexistente), help("{ayuda}"))]
    ExportInexistente {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("no existe")]
        span: SourceSpan,
        nombre: String,
        ayuda: String,
    },

    #[error("'{nombre}' ya está definido en este módulo.")]
    #[diagnostic(
        code(wn::modulo::nombre_en_uso),
        help("Elegí otro nombre para el import con `queri ... como otro_nombre`.")
    )]
    NombreEnUso {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("este nombre ya existe")]
        span: SourceSpan,
        nombre: String,
    },

    #[error("Falló un módulo importado (cadena de imports: {cadena})")]
    #[diagnostic(code(wn::modulo::error_en_modulo))]
    ErrorEnModulo {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("importado desde acá")]
        span: SourceSpan,
        cadena: String,
        #[diagnostic_source]
        causa: CausaDiagnostico,
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

    pub fn modulo_no_encontrado(
        source: &SourceFile,
        span: SourceSpan,
        modulo: impl Into<String>,
        ayuda: impl Into<String>,
    ) -> Self {
        Self::ModuloNoEncontrado {
            src: Arc::new(source.named_source()),
            span,
            modulo: modulo.into(),
            ayuda: ayuda.into(),
        }
    }

    pub fn modulo_ilegible(
        source: &SourceFile,
        span: SourceSpan,
        modulo: impl Into<String>,
        causa: impl Into<String>,
    ) -> Self {
        Self::ModuloIlegible {
            src: Arc::new(source.named_source()),
            span,
            modulo: modulo.into(),
            causa: causa.into(),
        }
    }

    pub fn import_circular(
        source: &SourceFile,
        span: SourceSpan,
        cadena: impl Into<String>,
    ) -> Self {
        Self::ImportCircular {
            src: Arc::new(source.named_source()),
            span,
            cadena: cadena.into(),
        }
    }

    pub fn modulo_conflicto(
        source: &SourceFile,
        span: SourceSpan,
        modulo: impl Into<String>,
        candidatos: impl Into<String>,
    ) -> Self {
        Self::ModuloConflicto {
            src: Arc::new(source.named_source()),
            span,
            modulo: modulo.into(),
            candidatos: candidatos.into(),
        }
    }

    pub fn modulo_reservado(
        source: &SourceFile,
        span: SourceSpan,
        nombre: impl Into<String>,
    ) -> Self {
        Self::ModuloReservado {
            src: Arc::new(source.named_source()),
            span,
            nombre: nombre.into(),
        }
    }

    pub fn no_exportado(
        source: &SourceFile,
        span: SourceSpan,
        modulo: impl Into<String>,
        nombre: impl Into<String>,
    ) -> Self {
        Self::NoExportado {
            src: Arc::new(source.named_source()),
            span,
            modulo: modulo.into(),
            nombre: nombre.into(),
        }
    }

    pub fn export_inexistente(
        source: &SourceFile,
        span: SourceSpan,
        nombre: impl Into<String>,
        ayuda: impl Into<String>,
    ) -> Self {
        Self::ExportInexistente {
            src: Arc::new(source.named_source()),
            span,
            nombre: nombre.into(),
            ayuda: ayuda.into(),
        }
    }

    pub fn nombre_en_uso(source: &SourceFile, span: SourceSpan, nombre: impl Into<String>) -> Self {
        Self::NombreEnUso {
            src: Arc::new(source.named_source()),
            span,
            nombre: nombre.into(),
        }
    }

    pub fn error_en_modulo(
        source: &SourceFile,
        span: SourceSpan,
        cadena: impl Into<String>,
        causa: WnDiagnostic,
    ) -> Self {
        Self::ErrorEnModulo {
            src: Arc::new(source.named_source()),
            span,
            cadena: cadena.into(),
            causa: CausaDiagnostico(Box::new(causa)),
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
            | Self::ModuloNoEncontrado { span, .. }
            | Self::ModuloIlegible { span, .. }
            | Self::ImportCircular { span, .. }
            | Self::ModuloConflicto { span, .. }
            | Self::ModuloReservado { span, .. }
            | Self::NoExportado { span, .. }
            | Self::ExportInexistente { span, .. }
            | Self::NombreEnUso { span, .. }
            | Self::ErrorEnModulo { span, .. }
            | Self::Runtime { span, .. } => Some(*span),
            Self::Interno { .. } => None,
        }
    }
}
