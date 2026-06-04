use std::{
    cell::RefCell,
    io::{self, Cursor, Write},
    rc::Rc,
    sync::Arc,
};

use wasm_bindgen::prelude::*;
use wn::{lexer::tokenizar, parser::parsear};
use wn_diagnostics::{SourceFile, WnDiagnostic};
use wn_vm::{compiler::compilar, vm::VM};

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn ejecutar(fuente: &str, stdin: &str) -> ResultadoEjecucion {
    let captura = CapturaSalida::default();

    let resultado = ejecutar_impl(fuente, stdin, captura.clone());
    match resultado {
        Ok(()) => ResultadoEjecucion::ok(captura.contenido()),
        Err(error) => ResultadoEjecucion::err(captura.contenido(), error),
    }
}

fn ejecutar_impl(fuente: &str, stdin: &str, captura: CapturaSalida) -> Result<(), DiagnosticoWasm> {
    let archivo = "<playground>";
    let source = Arc::new(SourceFile::new(archivo, fuente));
    let tokens = tokenizar(fuente).map_err(|err| DiagnosticoWasm::from_diagnostic(err, fuente))?;
    let stmts = parsear(tokens, fuente, archivo)
        .map_err(|err| DiagnosticoWasm::from_diagnostic(err, fuente))?;
    let chunk =
        compilar(&stmts, source).map_err(|err| DiagnosticoWasm::from_diagnostic(err, fuente))?;
    let mut vm = VM::con_io(captura, Cursor::new(stdin.as_bytes().to_vec()));
    vm.run(&chunk)
        .map(|_| ())
        .map_err(|err| DiagnosticoWasm::from_diagnostic(err, fuente))
}

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

#[wasm_bindgen]
pub struct ResultadoEjecucion {
    salida: String,
    error: Option<DiagnosticoWasm>,
}

impl ResultadoEjecucion {
    fn ok(salida: String) -> Self {
        Self {
            salida,
            error: None,
        }
    }

    fn err(salida: String, error: DiagnosticoWasm) -> Self {
        Self {
            salida,
            error: Some(error),
        }
    }
}

#[wasm_bindgen]
impl ResultadoEjecucion {
    #[wasm_bindgen(getter)]
    pub fn salida(&self) -> String {
        self.salida.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<DiagnosticoWasm> {
        self.error.clone()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct DiagnosticoWasm {
    fase: String,
    mensaje: String,
    offset: Option<usize>,
    len: Option<usize>,
    linea: Option<u32>,
}

impl DiagnosticoWasm {
    fn from_diagnostic(err: WnDiagnostic, fuente: &str) -> Self {
        let (fase, mensaje) = match &err {
            WnDiagnostic::Lexico { mensaje, .. } => ("lexico", mensaje.clone()),
            WnDiagnostic::Sintaxis { mensaje, .. } => ("sintaxis", mensaje.clone()),
            WnDiagnostic::Compilacion { mensaje, .. } => ("compilacion", mensaje.clone()),
            WnDiagnostic::Runtime { mensaje, .. } => ("runtime", mensaje.clone()),
            WnDiagnostic::Interno { mensaje } => ("interno", mensaje.clone()),
            _ => ("runtime", err.to_string()),
        };
        let (offset, len, linea) = match err.primary_span() {
            Some(span) => {
                let offset = span.offset();
                let len = span.len();
                (
                    Some(offset),
                    Some(len),
                    Some(line_for_offset(fuente, offset)),
                )
            }
            None => (None, None, None),
        };

        Self {
            fase: fase.to_string(),
            mensaje,
            offset,
            len,
            linea,
        }
    }
}

#[wasm_bindgen]
impl DiagnosticoWasm {
    #[wasm_bindgen(getter)]
    pub fn fase(&self) -> String {
        self.fase.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn mensaje(&self) -> String {
        self.mensaje.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    #[wasm_bindgen(getter)]
    pub fn len(&self) -> Option<usize> {
        self.len
    }

    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> Option<bool> {
        self.len.map(|len| len == 0)
    }

    #[wasm_bindgen(getter)]
    pub fn linea(&self) -> Option<u32> {
        self.linea
    }
}

fn line_for_offset(src: &str, offset: usize) -> u32 {
    let mut line = 1u32;
    for ch in src[..offset.min(src.len())].chars() {
        if ch == '\n' {
            line += 1;
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ejecuta_programa_y_captura_salida() {
        let resultado = ejecutar(r#"lorea("hola")"#, "");

        assert_eq!(resultado.salida(), "hola\n");
        assert!(resultado.error().is_none());
    }

    #[test]
    fn pregunta_consume_stdin_precargado() {
        let resultado = ejecutar(
            r#"
wea nombre = pregunta("Nombre: ")
lorea("Hola " + nombre)
"#,
            "Ada\n",
        );

        assert_eq!(resultado.salida(), "Nombre: Hola Ada\n");
        assert!(resultado.error().is_none());
    }

    #[test]
    fn pregunta_sin_stdin_suficiente_devuelve_runtime_explicito() {
        let resultado = ejecutar(
            r#"
lorea(pregunta("Uno: "))
lorea(pregunta("Dos: "))
"#,
            "primero\n",
        );

        assert_eq!(resultado.salida(), "Uno: primero\nDos: ");
        let error = resultado.error().expect("se esperaba error");
        assert_eq!(error.fase(), "runtime");
        assert_eq!(
            error.mensaje(),
            "pregunta() pidió más entrada que la provista."
        );
        assert_eq!(error.linea(), Some(3));
    }

    #[test]
    fn error_lexico_expone_datos_estructurados() {
        let resultado = ejecutar("@", "");

        let error = resultado.error().expect("se esperaba error");
        assert_eq!(error.fase(), "lexico");
        assert!(error.mensaje().contains("Carácter inesperado"));
        assert_eq!(error.offset(), Some(0));
        assert_eq!(error.len(), Some(1));
        assert_eq!(error.linea(), Some(1));
    }

    #[test]
    fn error_de_sintaxis_expone_datos_estructurados() {
        let resultado = ejecutar("wea x =", "");

        let error = resultado.error().expect("se esperaba error");
        assert_eq!(error.fase(), "sintaxis");
        assert!(error.mensaje().contains("Expresión inesperada"));
        assert_eq!(error.offset(), Some(7));
        assert_eq!(error.linea(), Some(1));
    }

    #[test]
    fn error_de_compilacion_expone_datos_estructurados() {
        let resultado = ejecutar("devolver 1", "");

        let error = resultado.error().expect("se esperaba error");
        assert_eq!(error.fase(), "compilacion");
        assert_eq!(
            error.mensaje(),
            "'devolver' solo puede usarse dentro de una pega papito."
        );
        assert_eq!(error.offset(), Some(0));
        assert_eq!(error.len(), Some(10));
        assert_eq!(error.linea(), Some(1));
    }

    #[test]
    fn runtime_preserva_salida_parcial_antes_del_fallo() {
        let resultado = ejecutar(
            r#"
lorea("antes")
numero("nope")
"#,
            "",
        );

        assert_eq!(resultado.salida(), "antes\n");
        let error = resultado.error().expect("se esperaba error");
        assert_eq!(error.fase(), "runtime");
        assert!(error.mensaje().contains("No pude convertir"));
        assert_eq!(error.linea(), Some(3));
    }
}
