use miette::NamedSource;

use crate::{
    ast::{Expr, OpBin, OpUn, Stmt},
    error::WnError,
    lexer::token::{Span, Token, TokenKind},
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    src: String,
    filename: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, src: &str, filename: &str) -> Self {
        Parser {
            tokens,
            pos: 0,
            src: src.to_string(),
            filename: filename.to_string(),
        }
    }

    fn make_source(&self) -> NamedSource<String> {
        NamedSource::new(&self.filename, self.src.clone())
    }

    fn error(&self, span: &Span, mensaje: impl Into<String>) -> WnError {
        WnError::Sintaxis {
            src: self.make_source(),
            span: span.into(),
            mensaje: mensaje.into(),
        }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_span(&self) -> &Span {
        &self.tokens[self.pos].span
    }

    fn peek_next(&self) -> &TokenKind {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1].kind
        } else {
            &TokenKind::EOF
        }
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn consume(&mut self, kind: &TokenKind) -> Result<&Token, WnError> {
        if self.peek() == kind {
            Ok(self.advance())
        } else {
            let span = self.peek_span().clone();
            Err(self.error(
                &span,
                format!("Esperaba {:?} pero encontré {:?}.", kind, self.peek()),
            ))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::EOF)
    }

    pub fn parsear(&mut self) -> Result<Vec<Stmt>, WnError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, WnError> {
        match self.peek() {
            TokenKind::Wea => self.parse_decl_wea(),
            TokenKind::Duro => self.parse_decl_duro(),
            TokenKind::Pega => self.parse_decl_pega(),
            TokenKind::Cachai => self.parse_cachai(),
            TokenKind::Mientras => self.parse_mientras(),
            TokenKind::Para => self.parse_para(),
            TokenKind::Ojo => self.parse_ojo(),
            TokenKind::Devolver => self.parse_devolver(),
            TokenKind::Cortala => {
                self.advance();
                Ok(Stmt::Cortala)
            }
            TokenKind::Sigue => {
                self.advance();
                Ok(Stmt::Sigue)
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_decl_wea(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        let nombre = self.expect_ident()?;
        self.consume(&TokenKind::Asignar)?;
        let valor = self.parse_expr()?;
        Ok(Stmt::DeclWea {
            nombre,
            valor,
            es_duro: false,
        })
    }

    fn parse_decl_duro(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        let nombre = self.expect_ident()?;
        self.consume(&TokenKind::Asignar)?;
        let valor = self.parse_expr()?;
        Ok(Stmt::DeclWea {
            nombre,
            valor,
            es_duro: true,
        })
    }

    fn parse_decl_pega(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        let nombre = self.expect_ident()?;
        self.consume(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.consume(&TokenKind::RParen)?;
        let cuerpo = self.parse_block()?;
        Ok(Stmt::DeclPega {
            nombre,
            params,
            cuerpo,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<String>, WnError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }
        params.push(self.expect_ident()?);
        while self.check(&TokenKind::Coma) {
            self.advance();
            params.push(self.expect_ident()?);
        }
        Ok(params)
    }

    fn parse_cachai(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        self.consume(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.consume(&TokenKind::RParen)?;
        let entonces = self.parse_block()?;
        let si_no = if self.check(&TokenKind::Si) && self.peek_next() == &TokenKind::No {
            self.advance();
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Stmt::Cachai {
            cond,
            entonces,
            si_no,
        })
    }

    fn parse_mientras(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        self.consume(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.consume(&TokenKind::RParen)?;
        let cuerpo = self.parse_block()?;
        Ok(Stmt::Mientras { cond, cuerpo })
    }

    fn parse_para(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        self.consume(&TokenKind::LParen)?;
        let var = self.expect_ident()?;
        self.consume(&TokenKind::En)?;
        let iterable = self.parse_expr()?;
        self.consume(&TokenKind::RParen)?;
        let cuerpo = self.parse_block()?;
        Ok(Stmt::Para {
            var,
            iterable,
            cuerpo,
        })
    }

    fn parse_ojo(&mut self) -> Result<Stmt, WnError> {
        self.advance();
        let cuerpo = self.parse_block()?;
        self.consume(&TokenKind::Cago)?;
        self.consume(&TokenKind::LParen)?;
        let error_var = self.expect_ident()?;
        self.consume(&TokenKind::RParen)?;
        let manejo = self.parse_block()?;
        Ok(Stmt::Ojo {
            cuerpo,
            error_var,
            manejo,
        })
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, WnError> {
        Ok(Stmt::Expresion(self.parse_expr()?))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, WnError> {
        self.consume(&TokenKind::LLave)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RLlave) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        self.consume(&TokenKind::RLlave)?;
        Ok(stmts)
    }

    fn parse_devolver(&mut self) -> Result<Stmt, WnError> {
        self.advance();

        // `devolver` retorna nada implícitamente
        let valor = if matches!(self.peek(), TokenKind::RLlave | TokenKind::EOF) {
            Expr::Nada
        } else {
            self.parse_expr()?
        };

        Ok(Stmt::Devolver { valor })
    }

    fn parse_expr(&mut self) -> Result<Expr, WnError> {
        self.parse_pratt(0)
    }

    fn parse_pratt(&mut self, min_bp: u8) -> Result<Expr, WnError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::O => OpBin::O,
                TokenKind::Y => OpBin::Y,
                TokenKind::IgualIgual => OpBin::Eq,
                TokenKind::BangIgual => OpBin::Neq,
                TokenKind::Menor => OpBin::Lt,
                TokenKind::Mayor => OpBin::Gt,
                TokenKind::MenorIgual => OpBin::Lte,
                TokenKind::MayorIgual => OpBin::Gte,
                TokenKind::Mas => OpBin::Suma,
                TokenKind::Menos => OpBin::Resta,
                TokenKind::Estrella => OpBin::Mul,
                TokenKind::Diagonal => OpBin::Div,
                TokenKind::Modulo => OpBin::Mod,
                _ => break,
            };
            let (left_bp, right_bp) = infix_binding_power(&op);
            if left_bp < min_bp {
                break;
            }
            let span_start = self.peek_span().start;
            self.advance();
            let rhs = self.parse_pratt(right_bp)?;
            let span = Span::new(span_start, self.peek_span().start);
            lhs = Expr::Binario {
                izq: Box::new(lhs),
                op,
                der: Box::new(rhs),
                span,
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, WnError> {
        match self.peek() {
            TokenKind::No => {
                let span_start = self.peek_span().start;
                self.advance();
                let expr = self.parse_unary()?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Unario {
                    op: OpUn::No,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Menos => {
                let span_start = self.peek_span().start;
                self.advance();
                let expr = self.parse_unary()?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Unario {
                    op: OpUn::Neg,
                    expr: Box::new(expr),
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, WnError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                TokenKind::LParen => {
                    let span_start = self.peek_span().start;
                    self.advance();
                    let args = self.parse_args()?;
                    self.consume(&TokenKind::RParen)?;
                    let span = Span::new(span_start, self.peek_span().start);
                    expr = Expr::Llamada {
                        callee: Box::new(expr),
                        args,
                        span,
                    };
                }
                TokenKind::LCorchete => {
                    let span_start = self.peek_span().start;
                    self.advance();
                    let indice = self.parse_expr()?;
                    self.consume(&TokenKind::RCorchete)?;
                    let span = Span::new(span_start, self.peek_span().start);
                    expr = Expr::Indice {
                        objeto: Box::new(expr),
                        indice: Box::new(indice),
                        span,
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, WnError> {
        let mut args = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while self.check(&TokenKind::Coma) {
            self.advance();
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, WnError> {
        match self.peek().clone() {
            TokenKind::Numero(n) => {
                self.advance();
                Ok(Expr::Numero(n))
            }
            TokenKind::Texto(s) => {
                self.advance();
                Ok(Expr::Texto(s))
            }
            TokenKind::Verdad => {
                self.advance();
                Ok(Expr::Booleano(true))
            }
            TokenKind::Falso => {
                self.advance();
                Ok(Expr::Booleano(false))
            }
            TokenKind::Nada => {
                self.advance();
                Ok(Expr::Nada)
            }

            TokenKind::Ident(nombre) => {
                let span = self.peek_span().clone();
                self.advance();
                if self.check(&TokenKind::Asignar) {
                    self.advance();
                    let valor = self.parse_expr()?;
                    let end = self.peek_span().start;
                    return Ok(Expr::Asignacion {
                        nombre,
                        valor: Box::new(valor),
                        span: Span::new(span.start, end),
                    });
                }
                Ok(Expr::Ident(nombre, span))
            }

            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(&TokenKind::RParen)?;
                Ok(expr)
            }

            TokenKind::LCorchete => {
                let span_start = self.peek_span().start;
                self.advance();
                let mut items = Vec::new();
                if !self.check(&TokenKind::RCorchete) {
                    items.push(self.parse_expr()?);
                    while self.check(&TokenKind::Coma) {
                        self.advance();
                        if self.check(&TokenKind::RCorchete) {
                            break;
                        }
                        items.push(self.parse_expr()?);
                    }
                }
                self.consume(&TokenKind::RCorchete)?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Lista(items, span))
            }

            TokenKind::LLave => {
                let span_start = self.peek_span().start;
                self.advance();
                let mut pairs = Vec::new();
                if !self.check(&TokenKind::RLlave) {
                    let key = self.parse_expr()?;
                    self.consume(&TokenKind::Colon)?;
                    let val = self.parse_expr()?;
                    pairs.push((key, val));
                    while self.check(&TokenKind::Coma) {
                        self.advance();
                        if self.check(&TokenKind::RLlave) {
                            break;
                        }
                        let key = self.parse_expr()?;
                        self.consume(&TokenKind::Colon)?;
                        let val = self.parse_expr()?;
                        pairs.push((key, val));
                    }
                }
                self.consume(&TokenKind::RLlave)?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Mapa(pairs, span))
            }

            other => {
                let span = self.peek_span().clone();
                Err(self.error(&span, format!("Expresión inesperada: {:?}.", other)))
            }
        }
    }

    fn expect_ident(&mut self) -> Result<String, WnError> {
        match self.peek().clone() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s)
            }
            other => {
                let span = self.peek_span().clone();
                Err(self.error(
                    &span,
                    format!(
                        "Esperaba un identificador pero encontré esta wea {:?}.",
                        other
                    ),
                ))
            }
        }
    }
}

fn infix_binding_power(op: &OpBin) -> (u8, u8) {
    match op {
        OpBin::O => (1, 2),
        OpBin::Y => (3, 4),
        OpBin::Eq | OpBin::Neq => (5, 6),
        OpBin::Lt | OpBin::Gt | OpBin::Lte | OpBin::Gte => (7, 8),
        OpBin::Suma | OpBin::Resta => (9, 10),
        OpBin::Mul | OpBin::Div | OpBin::Mod => (11, 12),
    }
}

pub fn parsear(tokens: Vec<Token>, src: &str, filename: &str) -> Result<Vec<Stmt>, WnError> {
    Parser::new(tokens, src, filename).parsear()
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;

    use super::*;
    use crate::lexer::tokenizar;

    fn parse_program(src: &str) -> Vec<Stmt> {
        let tokens = tokenizar(src).unwrap();
        parsear(tokens, src, "<test>").unwrap()
    }

    #[test]
    fn ast_snapshot_declaracion_duro() {
        let ast = parse_program("duro PI = 3.14");

        assert_debug_snapshot!(ast);
    }

    #[test]
    fn ast_snapshot_declaracion_wea() {
        let ast = parse_program("wea x = 10 + 20");

        assert_debug_snapshot!(ast);
    }
}
