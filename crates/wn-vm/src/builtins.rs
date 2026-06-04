use crate::{
    native::{NativeContext, NativeFn},
    value::Value,
    vm::VmError,
};
use std::{io::Write, rc::Rc};

/// Slice estático de todas las nativas core.
/// El VM itera esto en `registrar_nativas` para cargarlas en el entorno global.
pub static NATIVAS_CORE: &[NativeFn] = &[
    NativeFn {
        nombre: "lorea",
        aridad: None,
        func: lorea,
    },
    NativeFn {
        nombre: "largo",
        aridad: Some(1),
        func: largo,
    },
    NativeFn {
        nombre: "cachar",
        aridad: Some(1),
        func: cachar,
    },
    NativeFn {
        nombre: "pregunta",
        aridad: Some(1),
        func: pregunta,
    },
    NativeFn {
        nombre: "numero",
        aridad: Some(1),
        func: numero,
    },
    NativeFn {
        nombre: "texto",
        aridad: Some(1),
        func: texto,
    },
];

fn lorea(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let linea = args
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    writeln!(ctx.salida.borrow_mut(), "{linea}")
        .map_err(|e| VmError::TipoInvalido(format!("Error escribiendo output: {e}")))?;
    Ok(Value::Nada)
}

fn largo(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Texto(s) => Ok(Value::Numero(s.chars().count() as f64)),
        Value::Lista(xs) => Ok(Value::Numero(xs.borrow().len() as f64)),
        other => Err(VmError::TipoInvalido(format!(
            "largo() solo funciona con texto o lista, no con '{}'.",
            other.tipo_nombre()
        ))),
    }
}

fn cachar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::Texto(Rc::from(args[0].tipo_nombre())))
}

fn pregunta(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    write!(ctx.salida.borrow_mut(), "{}", args[0])
        .map_err(|e| VmError::TipoInvalido(format!("Error escribiendo output: {e}")))?;
    ctx.salida.borrow_mut().flush().ok();

    let mut input = String::new();
    let leidos = ctx
        .entrada
        .borrow_mut()
        .read_line(&mut input)
        .map_err(|e| VmError::TipoInvalido(format!("Error leyendo input: {e}")))?;
    if leidos == 0 {
        return Err(VmError::EntradaAgotada);
    }
    Ok(Value::Texto(Rc::from(
        input.trim_end_matches('\n').trim_end_matches('\r'),
    )))
}

fn numero(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Numero(n) => Ok(Value::Numero(*n)),
        Value::Texto(s) => parsear_numero(s),
        other => Err(VmError::TipoInvalido(format!(
            "numero() solo convierte textos o números, no un '{}'.",
            other.tipo_nombre()
        ))),
    }
}

fn texto(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    Ok(Value::Texto(Rc::from(args[0].to_string().as_str())))
}

pub(crate) fn parsear_numero(s: &str) -> Result<Value, VmError> {
    let limpio = s.trim();
    if limpio.is_empty() || !es_numero_simple(limpio) {
        return Err(VmError::TextoNoConvertibleANumero(s.to_string()));
    }
    let n = limpio.parse::<f64>()
        .map_err(|_| VmError::TextoNoConvertibleANumero(s.to_string()))?;
    if !n.is_finite() {
        return Err(VmError::TextoNoConvertibleANumero(s.to_string()));
    }
    Ok(Value::Numero(n))
}

fn es_numero_simple(s: &str) -> bool {
    let sin_signo = match s.strip_prefix('-') {
        Some(resto) if !resto.is_empty() => resto,
        Some(_) => return false, // solo "-"
        None => s,
    };
    let mut partes = sin_signo.split('.');
    let entero = partes.next().unwrap_or_default();
    let decimal = partes.next();
    if partes.next().is_some()          // más de un punto
        || entero.is_empty()
        || !entero.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    match decimal {
        Some(frac) => !frac.is_empty() && frac.chars().all(|c| c.is_ascii_digit()),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        io::{Cursor, Write},
        rc::Rc,
    };

    use super::*;
    use crate::{native::NativeContext, vm::VmError};

    type BoxWrite = RefCell<Box<dyn Write>>;
    type BoxRead = RefCell<Box<dyn std::io::BufRead>>;
    type NullIo = (BoxWrite, BoxRead);
    type CaptureIo = (Rc<RefCell<Vec<u8>>>, BoxWrite, BoxRead);

    /// I/O desechable para funciones que no usan salida/entrada.
    fn null_io() -> NullIo {
        (
            RefCell::new(Box::new(Vec::<u8>::new())),
            RefCell::new(Box::new(Cursor::new(Vec::<u8>::new()))),
        )
    }

    /// I/O con entrada controlada (para `pregunta`).
    fn io_con_entrada(input: &[u8]) -> NullIo {
        (
            RefCell::new(Box::new(Vec::<u8>::new())),
            RefCell::new(Box::new(Cursor::new(input.to_vec()))),
        )
    }

    /// Buffer de captura: permite leer lo que se escribió después de la llamada.
    ///
    /// El truco: `Rc<RefCell<Vec<u8>>>` queda en scope del test,
    /// el `Box<dyn Write>` solo tiene una referencia clonada.
    struct CaptureBuf(Rc<RefCell<Vec<u8>>>);
    impl Write for CaptureBuf {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Devuelve (buffer compartido, salida boxeada, entrada vacía).
    fn io_con_captura() -> CaptureIo {
        let buf = Rc::new(RefCell::new(Vec::new()));
        let salida = RefCell::new(Box::new(CaptureBuf(Rc::clone(&buf))) as Box<dyn Write>);
        let entrada =
            RefCell::new(Box::new(Cursor::new(Vec::<u8>::new())) as Box<dyn std::io::BufRead>);
        (buf, salida, entrada)
    }

    /// Construye el NativeContext desde las referencias de I/O del test.
    fn ctx<'a>(
        w: &'a RefCell<Box<dyn Write>>,
        r: &'a RefCell<Box<dyn std::io::BufRead>>,
    ) -> NativeContext<'a> {
        NativeContext {
            salida: w,
            entrada: r,
        }
    }

    #[test]
    fn nativas_core_registra_todos_los_nombres() {
        let nombres: Vec<_> = NATIVAS_CORE.iter().map(|n| n.nombre).collect();
        // Si agregamos una nativa y olvidamos registrarla, este test lo atrapa
        for esperado in ["lorea", "largo", "cachar", "pregunta", "numero", "texto"] {
            assert!(
                nombres.contains(&esperado),
                "falta '{esperado}' en NATIVAS_CORE"
            );
        }
    }

    #[test]
    fn nativas_core_sin_duplicados() {
        let nombres: Vec<_> = NATIVAS_CORE.iter().map(|n| n.nombre).collect();
        let mut unicos = nombres.clone();
        unicos.dedup();
        assert_eq!(
            nombres.len(),
            unicos.len(),
            "hay nombres duplicados en NATIVAS_CORE"
        );
    }

    #[test]
    fn largo_texto_ascii() {
        let (w, r) = null_io();
        let r = largo(&mut ctx(&w, &r), &[Value::Texto(Rc::from("hola"))]).unwrap();
        assert_eq!(r, Value::Numero(4.0));
    }

    #[test]
    fn largo_texto_unicode_cuenta_chars_no_bytes() {
        // "ñoño" = 4 chars, pero 6 bytes en UTF-8
        let (w, r) = null_io();
        let r = largo(&mut ctx(&w, &r), &[Value::Texto(Rc::from("ñoño"))]).unwrap();
        assert_eq!(r, Value::Numero(4.0));
    }

    #[test]
    fn largo_texto_vacio() {
        let (w, r) = null_io();
        let r = largo(&mut ctx(&w, &r), &[Value::Texto(Rc::from(""))]).unwrap();
        assert_eq!(r, Value::Numero(0.0));
    }

    #[test]
    fn largo_lista() {
        let (w, r) = null_io();
        let lista = Value::Lista(Rc::new(RefCell::new(vec![
            Value::Numero(1.0),
            Value::Numero(2.0),
            Value::Numero(3.0),
        ])));
        let r = largo(&mut ctx(&w, &r), &[lista]).unwrap();
        assert_eq!(r, Value::Numero(3.0));
    }

    #[test]
    fn largo_lista_vacia() {
        let (w, r) = null_io();
        let lista = Value::Lista(Rc::new(RefCell::new(vec![])));
        let r = largo(&mut ctx(&w, &r), &[lista]).unwrap();
        assert_eq!(r, Value::Numero(0.0));
    }

    #[test]
    fn largo_tipo_invalido_da_error() {
        let (w, r) = null_io();
        let err = largo(&mut ctx(&w, &r), &[Value::Numero(5.0)]).unwrap_err();
        assert!(matches!(err, VmError::TipoInvalido(_)));
    }

    #[test]
    fn cachar_retorna_nombre_del_tipo() {
        let (w, r) = null_io();
        let lista = Value::Lista(Rc::new(RefCell::new(vec![])));
        let mapa = Value::Mapa(Rc::new(RefCell::new(std::collections::HashMap::new())));

        let casos: Vec<(Value, &str)> = vec![
            (Value::Numero(1.0), "numero"),
            (Value::Texto(Rc::from("hola")), "texto"),
            (Value::Booleano(true), "booleano"),
            (Value::Booleano(false), "booleano"),
            (Value::Nada, "nada"),
            (lista, "lista"),
            (mapa, "mapa"),
        ];

        for (valor, nombre_esperado) in casos {
            let resultado = cachar(&mut ctx(&w, &r), &[valor.clone()]).unwrap();
            assert_eq!(
                resultado,
                Value::Texto(Rc::from(nombre_esperado)),
                "cachar({:?}) debería retornar '{nombre_esperado}'",
                valor
            );
        }
    }

    #[test]
    fn numero_convierte_texto_entero() {
        let (w, r) = null_io();
        let r = numero(&mut ctx(&w, &r), &[Value::Texto(Rc::from("42"))]).unwrap();
        assert_eq!(r, Value::Numero(42.0));
    }

    #[test]
    fn numero_convierte_texto_decimal() {
        let (w, r) = null_io();
        let r = numero(&mut ctx(&w, &r), &[Value::Texto(Rc::from("1.5"))]).unwrap();
        assert_eq!(r, Value::Numero(1.5));
    }

    #[test]
    fn numero_convierte_texto_negativo() {
        let (w, r) = null_io();
        let r = numero(&mut ctx(&w, &r), &[Value::Texto(Rc::from("-7"))]).unwrap();
        assert_eq!(r, Value::Numero(-7.0));
    }

    #[test]
    fn numero_ignora_espacios_al_convertir() {
        let (w, r) = null_io();
        let r = numero(&mut ctx(&w, &r), &[Value::Texto(Rc::from("  99  "))]).unwrap();
        assert_eq!(r, Value::Numero(99.0));
    }

    #[test]
    fn numero_ya_es_numero_retorna_igual() {
        let (w, r) = null_io();
        let r = numero(&mut ctx(&w, &r), &[Value::Numero(9.0)]).unwrap();
        assert_eq!(r, Value::Numero(9.0));
    }

    #[test]
    fn numero_tipo_invalido_da_error() {
        let (w, r) = null_io();
        let err = numero(&mut ctx(&w, &r), &[Value::Booleano(true)]).unwrap_err();
        assert!(matches!(err, VmError::TipoInvalido(_)));
    }

    #[test]
    fn parsear_numero_rechaza_texto_vacio() {
        assert!(matches!(
            parsear_numero(""),
            Err(VmError::TextoNoConvertibleANumero(_))
        ));
    }

    #[test]
    fn parsear_numero_rechaza_notacion_cientifica() {
        // "1e5" parsea como f64 válido en Rust, pero wn++ lo rechaza por diseño
        assert!(matches!(
            parsear_numero("1e5"),
            Err(VmError::TextoNoConvertibleANumero(_))
        ));
    }

    #[test]
    fn parsear_numero_rechaza_formatos_invalidos() {
        let casos = [
            "+7",    // signo positivo explícito no soportado
            "1.2.3", // dos puntos decimales
            ".5",    // sin parte entera
            "5.",    // sin parte decimal
            "-",     // solo signo
            "NaN",   // que Rust parsea como f64::NAN
            "Inf",   // que Rust parsea como f64::INFINITY
        ];
        for caso in casos {
            assert!(
                matches!(
                    parsear_numero(caso),
                    Err(VmError::TextoNoConvertibleANumero(_))
                ),
                "se esperaba error para '{caso}' pero no lo hubo"
            );
        }
    }

    #[test]
    fn parsear_numero_acepta_formatos_validos() {
        assert_eq!(parsear_numero("0").unwrap(), Value::Numero(0.0));
        assert_eq!(parsear_numero("42").unwrap(), Value::Numero(42.0));
        assert_eq!(parsear_numero("-3").unwrap(), Value::Numero(-3.0));
        assert_eq!(parsear_numero("1.5").unwrap(), Value::Numero(1.5));
        assert_eq!(parsear_numero("-0.5").unwrap(), Value::Numero(-0.5));
    }

    #[test]
    fn texto_convierte_numero_entero_sin_decimal() {
        // Value::Numero(42.0) → "42", no "42.0"
        // Esto depende del Display de Value
        let (w, r) = null_io();
        let r = texto(&mut ctx(&w, &r), &[Value::Numero(42.0)]).unwrap();
        assert_eq!(r, Value::Texto(Rc::from("42")));
    }

    #[test]
    fn texto_convierte_numero_decimal() {
        let (w, r) = null_io();
        let r = texto(&mut ctx(&w, &r), &[Value::Numero(1.5)]).unwrap();
        assert_eq!(r, Value::Texto(Rc::from("1.5")));
    }

    #[test]
    fn texto_convierte_booleanos_a_keywords_del_lenguaje() {
        let (w, r) = null_io();
        let t = texto(&mut ctx(&w, &r), &[Value::Booleano(true)]).unwrap();
        let f = texto(&mut ctx(&w, &r), &[Value::Booleano(false)]).unwrap();
        // Los booleans usan los keywords del lenguaje, no "true"/"false"
        assert_eq!(t, Value::Texto(Rc::from("verdad")));
        assert_eq!(f, Value::Texto(Rc::from("falso")));
    }

    #[test]
    fn texto_convierte_nada() {
        let (w, r) = null_io();
        let r = texto(&mut ctx(&w, &r), &[Value::Nada]).unwrap();
        assert_eq!(r, Value::Texto(Rc::from("nada")));
    }

    #[test]
    fn lorea_retorna_nada() {
        let (w, r) = null_io();
        let r = lorea(&mut ctx(&w, &r), &[Value::Texto(Rc::from("wena"))]).unwrap();
        assert_eq!(r, Value::Nada);
    }

    #[test]
    fn lorea_escribe_texto_con_newline() {
        let (buf, w, r) = io_con_captura();
        lorea(&mut ctx(&w, &r), &[Value::Texto(Rc::from("hola mundo"))]).unwrap();
        let salida = String::from_utf8(buf.borrow().clone()).unwrap();
        assert_eq!(salida, "hola mundo\n");
    }

    #[test]
    fn lorea_multiples_args_separados_por_espacio() {
        let (buf, w, r) = io_con_captura();
        lorea(
            &mut ctx(&w, &r),
            &[
                Value::Texto(Rc::from("hola")),
                Value::Numero(42.0),
                Value::Booleano(true),
            ],
        )
        .unwrap();
        let salida = String::from_utf8(buf.borrow().clone()).unwrap();
        assert_eq!(salida, "hola 42 verdad\n"); // espacios entre args, igual que Python's print()
    }

    #[test]
    fn lorea_sin_args_escribe_linea_vacia() {
        let (buf, w, r) = io_con_captura();
        lorea(&mut ctx(&w, &r), &[]).unwrap();
        let salida = String::from_utf8(buf.borrow().clone()).unwrap();
        assert_eq!(salida, "\n");
    }

    #[test]
    fn pregunta_lee_linea_y_elimina_newline() {
        let (w, r) = io_con_entrada(b"Rodrigo\n");
        let r = pregunta(&mut ctx(&w, &r), &[Value::Texto(Rc::from("Nombre: "))]).unwrap();
        // El \n se elimina; el usuario solo recibe el texto
        assert_eq!(r, Value::Texto(Rc::from("Rodrigo")));
    }

    #[test]
    fn pregunta_sin_entrada_da_error_explicito() {
        let (w, r) = io_con_entrada(b""); // ← entrada vacía = EOF inmediato
        let err = pregunta(&mut ctx(&w, &r), &[Value::Texto(Rc::from("Prompt: "))]).unwrap_err();
        assert!(matches!(err, VmError::EntradaAgotada));
    }
}
