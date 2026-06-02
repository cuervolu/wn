//! Conjunto de instrucciones del VM de WN++.
//!
//! El formato de cada instrucción es:
//! ```text
//! [OpCode: 1 byte] [operand?: 1-2 bytes]
//! ```

/// Instrucciones del bytecode de WN++.
///
/// `#[repr(u8)]` garantiza que cada variante se mapea a un byte específico,
/// lo que permite serializar/deserializar el bytecode directamente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Literales
    /// `CONSTANTE <u16>` — push constants[idx]
    Constante,
    /// push Value::Nada
    Nada,
    /// push Value::Booleano(true)
    Verdad,
    /// push Value::Booleano(false)
    Falso,

    // Stack
    /// Descarta el tope del stack.
    Pop,

    // Variables globales
    /// `DEFINIR_GLOBAL <u16>` — globals[name] = pop(); name = constants[idx]
    DefinirGlobal,
    /// `OBTENER_GLOBAL <u16>` — push globals[constants[idx]]
    ObtenerGlobal,
    /// `ASIGNAR_GLOBAL <u16>` — globals[constants[idx]] = peek() (sin pop)
    AsignarGlobal,

    // Variables locales (offsets en el frame actual)
    /// `OBTENER_LOCAL <u8>` — push stack[frame.base_slot + idx]
    ObtenerLocal,
    /// `ASIGNAR_LOCAL <u8>` — stack[frame.base_slot + idx] = peek() (sin pop)
    AsignarLocal,

    // Aritmética
    Suma,
    Resta,
    Mul,
    Div,
    Mod,
    /// Negación unaria `-x`.
    Neg,

    // Lógica
    /// NOT unario.
    No,

    // Comparación
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,

    // Control de flujo
    /// `SALTAR <u16>` — ip += offset (incondicional, hacia adelante)
    Saltar,
    /// `SALTAR_SI_FALSO <u16>` — if !peek(): ip += offset (no hace pop)
    SaltarSiFalso,
    /// `LOOP <u16>` — ip -= offset (salto hacia atrás para `mientras`)
    Loop,

    //Funciones
    /// `LLAMAR <u8>` — invoca la función con n_args argumentos
    Llamar,
    /// Retorna el tope del stack al frame anterior.
    Devolver,

    // Colecciones
    /// `CONSTRUIR_LISTA <u16>` — pop n valores → push Lista
    ConstruirLista,
    /// `CONSTRUIR_MAPA <u16>` — pop 2*n valores (clave, valor) → push Mapa
    ConstruirMapa,
    /// pop índice, pop objeto → push objeto[índice]
    ObtenerIndice,
    /// pop valor, pop índice, pop objeto → objeto[índice] = valor
    AsignarIndice,

    // I/O
    /// Imprime y descarta el tope del stack.
    Lorea,

    // Fin de ejecución
    /// Retorno implícito al final de un script de nivel raíz.
    RetornarNada,
}

impl TryFrom<u8> for OpCode {
    type Error = u8;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        // OpCode es #[repr(u8)]. Verificamos que `byte` esté dentro
        // del rango de discriminantes válidos antes de transmutarlo.
        if byte <= Self::RetornarNada as u8 {
            Ok(unsafe { std::mem::transmute::<u8, Self>(byte) })
        } else {
            Err(byte)
        }
    }
}
