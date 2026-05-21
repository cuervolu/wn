use miette::SourceSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}

impl From<&Span> for SourceSpan {
    fn from(s: &Span) -> Self {
        SourceSpan::new(s.start.into(), s.end - s.start)
    }
}

impl From<Span> for SourceSpan {
    fn from(s: Span) -> Self {
        (&s).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Token {
            kind,
            span: Span::new(start, end),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Numero(f64),
    Texto(String),
    Verdad,
    Falso,
    Nada,

    // Identifier
    Ident(String),

    // Declaration keywords
    Wea,
    Duro,
    Pega,

    // Control flow keywords
    Cachai,
    Si,
    Mientras,
    Para,
    En,

    // Error handling
    Ojo,
    Cago,

    Devolver,

    // Logical operators (keyword form)
    Y,
    O,
    No,

    // Arithmetic operators
    Mas,
    Menos,
    Estrella,
    Diagonal,
    Modulo,

    // Comparison operators
    IgualIgual,
    BangIgual,
    Menor,
    Mayor,
    MenorIgual,
    MayorIgual,

    // Assignment
    Asignar,

    // Delimiters
    LParen,
    RParen,
    LLave,
    RLlave,
    LCorchete,
    RCorchete,
    Coma,
    Colon,

    Cortala,
    Sigue,

    // End of file
    EOF,
}
