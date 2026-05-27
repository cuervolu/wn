use crate::interpreter::error::RuntimeError;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum WnError {
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

    #[error("La wea '{nombre}' no existe papito.")]
    #[diagnostic(
        code(wn::runtime::var_no_definida),
        help("¿Escribiste bien el nombre? Las variables se declaran con `wea {nombre} = valor`.")
    )]
    VarNoDefinida { nombre: String },

    #[error("Oe, '{nombre}' es duro, no lo podí cambiar.")]
    #[diagnostic(
        code(wn::runtime::constante_inmutable),
        help("Si necesitái cambiar el valor, declarálo con `wea` en vez de `duro`.")
    )]
    ConstanteInmutable { nombre: String },

    #[error("Weon, no se puede dividir por cero.")]
    #[diagnostic(
        code(wn::runtime::division_por_cero),
        help(
            "Revisá que el divisor no pueda ser cero. Podés usar `cachai (divisor != 0)` antes de dividir."
        )
    )]
    DivisionPorCero,

    #[error("Te fuiste al chancho, el índice {indice} no existe en la lista (largo: {largo}).")]
    #[diagnostic(
        code(wn::runtime::indice_invalido),
        help("Los índices parten de 0. El último elemento está en la posición {largo} - 1.")
    )]
    IndiceInvalido { indice: i64, largo: usize },

    #[error("La clave '{clave}' no existe en el mapa papito.")]
    #[diagnostic(
        code(wn::runtime::clave_inexistente),
        help("Revisá las claves del mapa antes de acceder. Podés usar `cachar()` para debuggear.")
    )]
    ClaveInexistente { clave: String },

    #[error("'{nombre}' no es una pega papito.")]
    #[diagnostic(
        code(wn::runtime::no_llamable),
        help(
            "Solo podís llamar pegas declaradas con `pega` o lambdas. Usá `cachar({nombre})` para ver su tipo."
        )
    )]
    NoLlamable { nombre: String },

    #[error("La pega espera {esperados} argumento(s), le pasaste {recibidos}.")]
    #[diagnostic(
        code(wn::runtime::num_arg_invalido),
        help("Revisá la firma de la pega y ajustá los argumentos que le estái pasando.")
    )]
    NumArgInvalido { esperados: usize, recibidos: usize },

    #[error("Error de tipos: {mensaje}")]
    #[diagnostic(
        code(wn::runtime::tipo_invalido),
        help("Usá `cachar(valor)` para ver el tipo de una variable antes de operar con ella.")
    )]
    TipoInvalido { mensaje: String },

    #[error("Error en tiempo de ejecución: {mensaje}")]
    #[diagnostic(code(wn::runtime), help("Revisa la lógica de tu programa ctm."))]
    Runtime { mensaje: String },
}

impl From<RuntimeError> for WnError {
    fn from(e: RuntimeError) -> Self {
        match e {
            RuntimeError::VarNoDefinida(nombre) => WnError::VarNoDefinida { nombre },
            RuntimeError::ConstanteInmutable(nombre) => WnError::ConstanteInmutable { nombre },
            RuntimeError::DivisionPorCero => WnError::DivisionPorCero,
            RuntimeError::IndiceInvalido(indice, largo) => {
                WnError::IndiceInvalido { indice, largo }
            }
            RuntimeError::ClaveInexistente(clave) => WnError::ClaveInexistente { clave },
            RuntimeError::NoLlamable(nombre) => WnError::NoLlamable { nombre },
            RuntimeError::NumArgInvalido(esperados, recibidos) => WnError::NumArgInvalido {
                esperados,
                recibidos,
            },
            RuntimeError::TipoInvalido(mensaje) => WnError::TipoInvalido { mensaje },
            // Estos tres son errores de control de flujo que no deberían
            // llegar al usuario, si llegan, es un bug en el intérprete
            RuntimeError::Retorno(_) => WnError::Runtime {
                mensaje: "'devolver' solo puede usarse dentro de una pega papito.".to_string(),
            },
            RuntimeError::Cortala => WnError::Runtime {
                mensaje: "'cortala' solo tiene sentido dentro de un bucle, po.".to_string(),
            },
            RuntimeError::Sigue => WnError::Runtime {
                mensaje: "'sigue' solo tiene sentido dentro de un bucle, compare.".to_string(),
            },
            RuntimeError::ErrorCatcheable(msg) => WnError::Runtime { mensaje: msg },
        }
    }
}
