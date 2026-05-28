use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

use wn::{
    error::WnError,
    interpreter::{Interprete, value::Valor},
    lexer::tokenizar,
    parser::parsear,
};

#[derive(Clone, Default)]
struct CapturaSalida {
    bytes: Rc<RefCell<Vec<u8>>>,
}

impl CapturaSalida {
    fn contenido(&self) -> String {
        String::from_utf8(self.bytes.borrow().clone()).expect("salida UTF-8 valida")
    }
}

impl Write for CapturaSalida {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.bytes.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn run_program(src: &str) -> Result<Valor, WnError> {
    let tokens = tokenizar(src)?;
    let stmts = parsear(tokens, src, "<test>")?;
    let mut interprete = Interprete::nuevo();
    interprete.correr(&stmts)
}

pub fn run_program_with_output(src: &str) -> (Result<Valor, WnError>, String) {
    let captura = CapturaSalida::default();
    let tokens = match tokenizar(src) {
        Ok(tokens) => tokens,
        Err(err) => return (Err(err), captura.contenido()),
    };
    let stmts = match parsear(tokens, src, "<test>") {
        Ok(stmts) => stmts,
        Err(err) => return (Err(err), captura.contenido()),
    };
    let mut interprete = Interprete::con_salida(captura.clone());
    let resultado = interprete.correr(&stmts);
    (resultado, captura.contenido())
}

pub fn run_program_with_io(src: &str, input: &str) -> (Result<Valor, WnError>, String) {
    let captura = CapturaSalida::default();
    let tokens = match tokenizar(src) {
        Ok(tokens) => tokens,
        Err(err) => return (Err(err), captura.contenido()),
    };
    let stmts = match parsear(tokens, src, "<test>") {
        Ok(stmts) => stmts,
        Err(err) => return (Err(err), captura.contenido()),
    };
    let mut interprete =
        Interprete::con_io(captura.clone(), io::Cursor::new(input.as_bytes().to_vec()));
    let resultado = interprete.correr(&stmts);
    (resultado, captura.contenido())
}

pub fn render_error(err: &WnError) -> String {
    err.to_string()
}
