use crate::lexer::token::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Numero(f64, Span),
    Texto(String, Span),
    Booleano(bool, Span),
    Nada(Span),

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
    AsignacionIndice {
        objeto: Box<Expr>,
        indice: Box<Expr>,
        valor: Box<Expr>,
        span: Span,
    },
    /// `texto::dividir` acceso calificado a un campo de módulo.
    ///
    /// El compilador convierte esto en `ObtenerPath`.
    /// Los segmentos son `["texto", "dividir"]`.
    PathAccess {
        segmentos: Vec<String>,
        span: Span,
    },
}

/// Qué se importa de un módulo.
#[derive(Debug, Clone)]
pub enum ImportItems {
    /// `queri texto` importa el módulo completo bajo su nombre (o alias).
    Todo,
    /// `queri std::{texto, lista}` importa submódulos específicos.
    Selectivo(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expresion {
        expr: Expr,
        span: Span,
    },

    DeclWea {
        nombre: String,
        valor: Expr,
        es_duro: bool,
        span: Span,
    },

    /// `queri texto`
    /// `queri std::{texto, lista}`
    /// `queri utils como u`
    Importar {
        /// Segmentos del path: `["std", "texto"]` o `["utils"]`.
        path: Vec<String>,
        items: ImportItems,
        /// Nombre con el que se vincula el módulo en el scope.
        /// Calculado en el compilador; no necesita vivir en el AST.
        alias: Option<String>,
        span: Span,
    },

    DeclPega {
        nombre: String,
        params: Vec<String>,
        cuerpo: Vec<Stmt>,
        span: Span,
    },

    Cachai {
        cond: Expr,
        entonces: Vec<Stmt>,
        si_no: Option<Vec<Stmt>>,
        span: Span,
    },

    Mientras {
        cond: Expr,
        cuerpo: Vec<Stmt>,
        span: Span,
    },

    Para {
        var: String,
        iterable: Expr,
        cuerpo: Vec<Stmt>,
        span: Span,
    },

    Devolver {
        valor: Expr,
        span: Span,
    },

    Ojo {
        cuerpo: Vec<Stmt>,
        error_var: String,
        manejo: Vec<Stmt>,
        span: Span,
    },
    Cortala(Span),
    Sigue(Span),
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

impl Expr {
    pub fn span(&self) -> &Span {
        match self {
            Expr::Numero(_, span)
            | Expr::Texto(_, span)
            | Expr::Booleano(_, span)
            | Expr::Nada(span)
            | Expr::Ident(_, span)
            | Expr::Lista(_, span)
            | Expr::Mapa(_, span)
            | Expr::Binario { span, .. }
            | Expr::Unario { span, .. }
            | Expr::Llamada { span, .. }
            | Expr::Indice { span, .. }
            | Expr::Asignacion { span, .. } => span,
            Expr::PathAccess { span, .. } => span,
            Expr::AsignacionIndice { span, .. } => span,
        }
    }
}

impl Stmt {
    pub fn span(&self) -> &Span {
        match self {
            Stmt::Expresion { span, .. }
            | Stmt::DeclWea { span, .. }
            | Stmt::DeclPega { span, .. }
            | Stmt::Cachai { span, .. }
            | Stmt::Mientras { span, .. }
            | Stmt::Para { span, .. }
            | Stmt::Devolver { span, .. }
            | Stmt::Ojo { span, .. }
            | Stmt::Cortala(span)
            | Stmt::Sigue(span) => span,
            Stmt::Importar { span, .. } => span,
        }
    }
}
