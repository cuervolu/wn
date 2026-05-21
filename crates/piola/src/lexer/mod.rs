pub mod token;

use miette::{NamedSource, SourceSpan};

use crate::error::PiolaError;
pub use token::{Span, Token, TokenKind};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    src: String,
    filename: String,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Lexer {
            chars: src.chars().collect(),
            pos: 0,
            src: src.to_string(),
            filename: "<anónimo>".to_string(),
        }
    }

    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = filename.into();
        self
    }

    fn make_source(&self) -> NamedSource<String> {
        NamedSource::new(&self.filename, self.src.clone())
    }

    fn error(&self, span: impl Into<SourceSpan>, mensaje: impl Into<String>) -> PiolaError {
        PiolaError::Lexico {
            src: self.make_source(),
            span: span.into(),
            mensaje: mensaje.into(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn byte_pos(&self) -> usize {
        self.chars[..self.pos].iter().map(|c| c.len_utf8()).sum()
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self, quote: char, start: usize) -> Result<TokenKind, PiolaError> {
        let mut s = String::new();
        loop {
            match self.advance() {
                None => {
                    return Err(self.error(
                        SourceSpan::new(start.into(), 1usize),
                        "Texto sin cerrar, te faltó la comilla del final weon.",
                    ));
                }
                Some(c) if c == quote => break,
                Some('\\') => match self.advance() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('\\') => s.push('\\'),
                    Some('"') => s.push('"'),
                    Some('\'') => s.push('\''),
                    Some(c) => {
                        s.push('\\');
                        s.push(c);
                    }
                    None => {
                        let pos = self.byte_pos();
                        return Err(self.error(
                            SourceSpan::new(pos.into(), 0usize),
                            "Escape incompleto al final del archivo.",
                        ));
                    }
                },
                Some(c) => s.push(c),
            }
        }
        Ok(TokenKind::Texto(s))
    }

    fn read_number(&mut self, first: char) -> TokenKind {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
            s.push('.');
            self.advance();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    s.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
        }
        TokenKind::Numero(s.parse().unwrap())
    }

    fn read_ident_or_keyword(&mut self, first: char) -> TokenKind {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "wea" => TokenKind::Wea,
            "duro" => TokenKind::Duro,
            "pega" => TokenKind::Pega,
            "cachai" => TokenKind::Cachai,
            "si" => TokenKind::Si,
            "mientras" => TokenKind::Mientras,
            "para" => TokenKind::Para,
            "en" => TokenKind::En,
            "ojo" => TokenKind::Ojo,
            "cago" => TokenKind::Cago,
            "y" => TokenKind::Y,
            "o" => TokenKind::O,
            "no" => TokenKind::No,
            "verdad" => TokenKind::Verdad,
            "falso" => TokenKind::Falso,
            "nada" => TokenKind::Nada,
            "devolver" => TokenKind::Devolver,
            "cortala" => TokenKind::Cortala,
            "sigue" => TokenKind::Sigue,
            _ => TokenKind::Ident(s),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, PiolaError> {
        self.skip_whitespace_and_comments();
        let start = self.byte_pos();

        let c = match self.advance() {
            None => return Ok(Some(Token::new(TokenKind::EOF, start, start))),
            Some(c) => c,
        };

        let kind = match c {
            '+' => TokenKind::Mas,
            '-' => TokenKind::Menos,
            '*' => TokenKind::Estrella,
            '/' => TokenKind::Diagonal,
            '%' => TokenKind::Modulo,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LLave,
            '}' => TokenKind::RLlave,
            '[' => TokenKind::LCorchete,
            ']' => TokenKind::RCorchete,
            ',' => TokenKind::Coma,
            ':' => TokenKind::Colon,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::IgualIgual
                } else {
                    TokenKind::Asignar
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangIgual
                } else {
                    return Err(self.error(
                        SourceSpan::new(start.into(), 1usize),
                        "Carácter inesperado '!'. ¿Querías decir '!='?",
                    ));
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::MenorIgual
                } else {
                    TokenKind::Menor
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::MayorIgual
                } else {
                    TokenKind::Mayor
                }
            }
            '"' | '\'' => self.read_string(c, start)?,
            d if d.is_ascii_digit() => self.read_number(d),
            a if a.is_alphabetic() || a == '_' => self.read_ident_or_keyword(a),
            other => {
                return Err(self.error(
                    SourceSpan::new(start.into(), other.len_utf8()),
                    format!("Carácter inesperado '{other}'."),
                ));
            }
        };

        let end = self.byte_pos();
        Ok(Some(Token::new(kind, start, end)))
    }

    pub fn tokenizar(mut self) -> Result<Vec<Token>, PiolaError> {
        let mut tokens = Vec::new();
        while let Some(tok) = self.next_token()? {
            let eof = tok.kind == TokenKind::EOF;
            tokens.push(tok);
            if eof {
                break;
            }
        }
        Ok(tokens)
    }
}

pub fn tokenizar(src: &str) -> Result<Vec<Token>, PiolaError> {
    Lexer::new(src).tokenizar()
}

pub fn tokenizar_archivo(src: &str, filename: &str) -> Result<Vec<Token>, PiolaError> {
    Lexer::new(src).with_filename(filename).tokenizar()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokeniza_declaracion_simple() {
        let tokens = tokenizar("wea x = 10 + 20").unwrap();
        let kinds: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        assert_eq!(kinds[0], &TokenKind::Wea);
        assert!(matches!(kinds[1], TokenKind::Ident(s) if s == "x"));
        assert_eq!(kinds[2], &TokenKind::Asignar);
        assert!(matches!(kinds[3], TokenKind::Numero(n) if *n == 10.0));
        assert_eq!(kinds[4], &TokenKind::Mas);
        assert!(matches!(kinds[5], TokenKind::Numero(n) if *n == 20.0));
        assert_eq!(kinds[6], &TokenKind::EOF);
    }

    #[test]
    fn tokeniza_string_con_comillas_dobles() {
        let tokens = tokenizar(r#"altiro("hola")"#).unwrap();
        assert!(matches!(&tokens[0].kind, TokenKind::Ident(s) if s == "altiro"));
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert!(matches!(&tokens[2].kind, TokenKind::Texto(s) if s == "hola"));
        assert_eq!(tokens[3].kind, TokenKind::RParen);
    }

    #[test]
    fn tokeniza_keywords_control_flujo() {
        let tokens = tokenizar("cachai si mientras para en").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Cachai);
        assert_eq!(tokens[1].kind, TokenKind::Si);
        assert_eq!(tokens[2].kind, TokenKind::Mientras);
        assert_eq!(tokens[3].kind, TokenKind::Para);
        assert_eq!(tokens[4].kind, TokenKind::En);
    }

    #[test]
    fn ignora_comentarios() {
        let tokens = tokenizar("wea x = 1 // esto es un comentario\nwea y = 2").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Wea);
        assert!(matches!(&tokens[1].kind, TokenKind::Ident(s) if s == "x"));
        assert_eq!(tokens[4].kind, TokenKind::Wea);
    }
}
