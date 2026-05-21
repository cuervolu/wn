use crate::lexer::token::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Numero(f64),
    Texto(String),
    Booleano(bool),
    Nada,

    Ident(String, Span),

    Binario {
        izq: Box<Expr>,
        op: OpBin,
        der: Box<Expr>,
        span: Span,
    },

    Unario {
        op: OpUn,
        expr: Box<Expr>,
        span: Span,
    },

    Llamada {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    Indice {
        objeto: Box<Expr>,
        indice: Box<Expr>,
        span: Span,
    },

    Lista(Vec<Expr>, Span),

    Mapa(Vec<(Expr, Expr)>, Span),

    Asignacion {
        nombre: String,
        valor: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expresion(Expr),

    DeclWea {
        nombre: String,
        valor: Expr,
        es_duro: bool,
    },

    DeclPega {
        nombre: String,
        params: Vec<String>,
        cuerpo: Vec<Stmt>,
    },

    Cachai {
        cond: Expr,
        entonces: Vec<Stmt>,
        si_no: Option<Vec<Stmt>>,
    },

    Mientras {
        cond: Expr,
        cuerpo: Vec<Stmt>,
    },

    Para {
        var: String,
        iterable: Expr,
        cuerpo: Vec<Stmt>,
    },

    Devolver {
        valor: Expr,
    },

    Ojo {
        cuerpo: Vec<Stmt>,
        error_var: String,
        manejo: Vec<Stmt>,
    },
    Cortala,
    Sigue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpBin {
    Suma,
    Resta,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Y,
    O,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpUn {
    No,
    Neg,
}
