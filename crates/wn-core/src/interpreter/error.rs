use crate::interpreter::value::Valor;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error("{0}")]
    TipoInvalido(String),

    #[error("La wea '{0}' no existe papito.")]
    VarNoDefinida(String),

    #[error("Te fuiste al chancho, el índice {0} no existe en la lista (largo: {1}).")]
    IndiceInvalido(i64, usize),

    #[error("La clave '{0}' no existe papito.")]
    ClaveInexistente(String),

    #[error("Oe, '{0}' es duro, no lo podí cambiar.")]
    ConstanteInmutable(String),

    #[error("Weon, no se puede dividir por cero.")]
    DivisionPorCero,

    #[error("La pega espera {0} args, le pasaste {1}.")]
    NumArgInvalido(usize, usize),

    #[error("'{0}' no es una pega papito.")]
    NoLlamable(String),

    #[error("No pude convertir {0:?} a número.")]
    TextoNoConvertibleANumero(String),

    #[error("devolver")]
    Retorno(Valor),

    #[error("{0}")]
    ErrorCatcheable(String),

    #[error("cortala")] // nunca se muestra al usuario
    Cortala,

    #[error("sigue")]
    Sigue,
}
