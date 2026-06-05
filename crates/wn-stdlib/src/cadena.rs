//! Módulo `cadena` de la biblioteca estándar.
//!
//! Funciones para manipulación de cadenas. Se acceden desde WN++ como:
//! ```text
//! queri cadena
//! cadena::dividir("hola wn++", " ")   // → ["hola", "wn++"]
//! cadena::mayusculas("hola")           // → "HOLA"
//! ```

use std::rc::Rc;
use wn_vm::{
    native::{NativeContext, NativeFn},
    value::Value,
    vm::VmError,
};

/// Todas las funciones del módulo `texto`.
pub static CADENA: &[NativeFn] = &[
    NativeFn {
        nombre: "dividir",
        aridad: Some(2),
        func: dividir,
    },
    NativeFn {
        nombre: "unir",
        aridad: Some(2),
        func: unir,
    },
    NativeFn {
        nombre: "recortar",
        aridad: Some(1),
        func: recortar,
    },
    NativeFn {
        nombre: "contiene",
        aridad: Some(2),
        func: contiene,
    },
    NativeFn {
        nombre: "empieza_con",
        aridad: Some(2),
        func: empieza_con,
    },
    NativeFn {
        nombre: "termina_con",
        aridad: Some(2),
        func: termina_con,
    },
    NativeFn {
        nombre: "mayusculas",
        aridad: Some(1),
        func: mayusculas,
    },
    NativeFn {
        nombre: "minusculas",
        aridad: Some(1),
        func: minusculas,
    },
    NativeFn {
        nombre: "reemplazar",
        aridad: Some(3),
        func: reemplazar,
    },
    NativeFn {
        nombre: "repetir",
        aridad: Some(2),
        func: repetir,
    },
    NativeFn {
        nombre: "indice_de",
        aridad: Some(2),
        func: indice_de,
    },
];

/// `dividir(s, sep) → lista` — divide `s` por el separador.
///
/// `dividir("a,b,c", ",")` → `["a", "b", "c"]`
fn dividir(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let (s, sep) = esperar_dos_textos("dividir", args)?;
    let partes = s
        .split(sep.as_ref())
        .map(|p| Value::Texto(Rc::from(p)))
        .collect();
    Ok(Value::Lista(Rc::new(std::cell::RefCell::new(partes))))
}

/// `unir(lista, sep) → texto` — une los elementos de la lista con el separador.
///
/// `unir(["a", "b", "c"], "-")` → `"a-b-c"`
/// Los elementos que no son texto se convierten con su representación natural.
fn unir(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match (&args[0], &args[1]) {
        (Value::Lista(xs), Value::Texto(sep)) => {
            let resultado = xs
                .borrow()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(sep.as_ref());
            Ok(Value::Texto(Rc::from(resultado.as_str())))
        }
        (a, b) => Err(tipo_invalido("unir", &[a, b], &["lista", "texto"])),
    }
}

/// `recortar(s) → cadena` — elimina espacios (y saltos de línea) al inicio y al final.
///
/// `recortar("  hola  ")` → `"hola"`
fn recortar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let s = esperar_texto("recortar", &args[0])?;
    Ok(Value::Texto(Rc::from(s.trim())))
}

/// `contiene(s, sub) → booleano` — indica si `sub` aparece dentro de `s`.
fn contiene(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let (s, sub) = esperar_dos_textos("contiene", args)?;
    Ok(Value::Booleano(s.contains(sub.as_ref())))
}

/// `empieza_con(s, prefijo) → booleano`
fn empieza_con(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let (s, prefijo) = esperar_dos_textos("empieza_con", args)?;
    Ok(Value::Booleano(s.starts_with(prefijo.as_ref())))
}

/// `termina_con(s, sufijo) → booleano`
fn termina_con(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let (s, sufijo) = esperar_dos_textos("termina_con", args)?;
    Ok(Value::Booleano(s.ends_with(sufijo.as_ref())))
}

/// `mayusculas(s) → texto` — convierte a mayúsculas usando las reglas de Unicode.
fn mayusculas(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let s = esperar_texto("mayusculas", &args[0])?;
    Ok(Value::Texto(Rc::from(s.to_uppercase().as_str())))
}

/// `minusculas(s) → texto` — convierte a minúsculas usando las reglas de Unicode.
fn minusculas(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let s = esperar_texto("minusculas", &args[0])?;
    Ok(Value::Texto(Rc::from(s.to_lowercase().as_str())))
}

/// `reemplazar(s, viejo, nuevo) → texto` — reemplaza todas las ocurrencias de `viejo` por `nuevo`.
///
/// `reemplazar("hola mundo", "mundo", "wn")` → `"hola wn"`
fn reemplazar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Texto(s), Value::Texto(viejo), Value::Texto(nuevo)) => Ok(Value::Texto(Rc::from(
            s.replace(viejo.as_ref(), nuevo.as_ref()).as_str(),
        ))),
        (a, b, c) => Err(tipo_invalido(
            "reemplazar",
            &[a, b, c],
            &["texto", "texto", "texto"],
        )),
    }
}

/// `repetir(s, n) → texto` — repite `s` exactamente `n` veces.
///
/// `repetir("ja", 3)` → `"jajaja"`
fn repetir(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match (&args[0], &args[1]) {
        (Value::Texto(s), Value::Numero(n)) => {
            let veces = numero_a_usize("repetir", *n)?;
            Ok(Value::Texto(Rc::from(s.repeat(veces).as_str())))
        }
        (a, b) => Err(tipo_invalido("repetir", &[a, b], &["texto", "numero"])),
    }
}

/// `indice_de(s, sub) → numero | nada` — posición de la primera aparición de `sub` en `s`.
///
/// Retorna el índice en caracteres (no bytes). Retorna `nada` si no se encuentra.
///
/// `indice_de("hola", "ol")` → `1`
/// `indice_de("hola", "z")` → `nada`
fn indice_de(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    let (s, sub) = esperar_dos_textos("indice_de", args)?;
    // Busca en bytes, convierte a índice de caracteres
    let resultado = s.find(sub.as_ref()).map(|byte_idx| {
        let char_idx = s[..byte_idx].chars().count();
        Value::Numero(char_idx as f64)
    });
    Ok(resultado.unwrap_or(Value::Nada))
}

fn esperar_texto<'a>(nombre: &str, valor: &'a Value) -> Result<&'a Rc<str>, VmError> {
    match valor {
        Value::Texto(s) => Ok(s),
        other => Err(VmError::TipoInvalido(format!(
            "{nombre}() espera texto, no '{}'.",
            other.tipo_nombre()
        ))),
    }
}

fn esperar_dos_textos<'a>(
    nombre: &str,
    args: &'a [Value],
) -> Result<(&'a Rc<str>, &'a Rc<str>), VmError> {
    match (&args[0], &args[1]) {
        (Value::Texto(a), Value::Texto(b)) => Ok((a, b)),
        (a, b) => Err(tipo_invalido(nombre, &[a, b], &["texto", "texto"])),
    }
}

fn numero_a_usize(nombre: &str, n: f64) -> Result<usize, VmError> {
    if !n.is_finite() || n.fract() != 0.0 || n < 0.0 {
        return Err(VmError::TipoInvalido(format!(
            "{nombre}() espera un número entero positivo, no {n}."
        )));
    }
    Ok(n as usize)
}

fn tipo_invalido(nombre: &str, recibidos: &[&Value], esperados: &[&str]) -> VmError {
    let rec = recibidos
        .iter()
        .map(|v| v.tipo_nombre())
        .collect::<Vec<_>>()
        .join(", ");
    let esp = esperados.join(", ");
    VmError::TipoInvalido(format!("{nombre}() espera ({esp}), recibió ({rec})."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::RefCell,
        io::{Cursor, Write},
        rc::Rc,
    };
    use wn_vm::vm::VmError;

    type BoxWrite = RefCell<Box<dyn Write>>;
    type BoxRead = RefCell<Box<dyn std::io::BufRead>>;
    type NullIo = (BoxWrite, BoxRead);

    fn null_io() -> NullIo {
        (
            RefCell::new(Box::new(Vec::<u8>::new())),
            RefCell::new(Box::new(Cursor::new(Vec::<u8>::new()))),
        )
    }

    fn ctx<'a>(w: &'a BoxWrite, r: &'a BoxRead) -> NativeContext<'a> {
        NativeContext {
            salida: w,
            entrada: r,
        }
    }

    fn txt(s: &str) -> Value {
        Value::Texto(Rc::from(s))
    }

    fn lista(xs: Vec<Value>) -> Value {
        Value::Lista(Rc::new(RefCell::new(xs)))
    }

    fn extraer_lista(v: Value) -> Vec<Value> {
        match v {
            Value::Lista(xs) => xs.borrow().clone(),
            other => panic!("se esperaba lista, se obtuvo {other:?}"),
        }
    }

    #[test]
    fn texto_registra_todas_las_funciones() {
        let nombres: Vec<_> = CADENA.iter().map(|f| f.nombre).collect();
        for esperado in [
            "dividir",
            "unir",
            "recortar",
            "contiene",
            "empieza_con",
            "termina_con",
            "mayusculas",
            "minusculas",
            "reemplazar",
            "repetir",
            "indice_de",
        ] {
            assert!(nombres.contains(&esperado), "falta '{esperado}' en TEXTO");
        }
    }

    #[test]
    fn dividir_por_espacio() {
        let (w, r) = null_io();
        let result = dividir(&mut ctx(&w, &r), &[txt("hola wn++"), txt(" ")]).unwrap();
        let partes = extraer_lista(result);
        assert_eq!(partes, vec![txt("hola"), txt("wn++")]);
    }

    #[test]
    fn dividir_separador_vacio_da_caracteres() {
        let (w, r) = null_io();
        let result = dividir(&mut ctx(&w, &r), &[txt("abc"), txt("")]).unwrap();
        // split("") en Rust da ["", "a", "b", "c", ""]
        let partes = extraer_lista(result);
        assert_eq!(partes.len(), 5); // comportamiento de str::split con ""
    }

    #[test]
    fn dividir_sin_separador_presente_retorna_lista_de_uno() {
        let (w, r) = null_io();
        let result = dividir(&mut ctx(&w, &r), &[txt("hola"), txt(",")]).unwrap();
        let partes = extraer_lista(result);
        assert_eq!(partes, vec![txt("hola")]);
    }

    #[test]
    fn dividir_tipo_invalido_da_error() {
        let (w, r) = null_io();
        let err = dividir(&mut ctx(&w, &r), &[Value::Numero(1.0), txt(",")]).unwrap_err();
        assert!(matches!(err, VmError::TipoInvalido(_)));
    }

    #[test]
    fn unir_lista_de_textos() {
        let (w, r) = null_io();
        let xs = lista(vec![txt("a"), txt("b"), txt("c")]);
        let result = unir(&mut ctx(&w, &r), &[xs, txt("-")]).unwrap();
        assert_eq!(result, txt("a-b-c"));
    }

    #[test]
    fn unir_convierte_numeros_a_texto() {
        let (w, r) = null_io();
        let xs = lista(vec![Value::Numero(1.0), Value::Numero(2.0)]);
        let result = unir(&mut ctx(&w, &r), &[xs, txt(", ")]).unwrap();
        assert_eq!(result, txt("1, 2"));
    }

    #[test]
    fn unir_lista_vacia_da_texto_vacio() {
        let (w, r) = null_io();
        let xs = lista(vec![]);
        let result = unir(&mut ctx(&w, &r), &[xs, txt(", ")]).unwrap();
        assert_eq!(result, txt(""));
    }

    #[test]
    fn recortar_elimina_espacios_extremos() {
        let (w, r) = null_io();
        let result = recortar(&mut ctx(&w, &r), &[txt("  hola  ")]).unwrap();
        assert_eq!(result, txt("hola"));
    }

    #[test]
    fn recortar_no_toca_el_centro() {
        let (w, r) = null_io();
        let result = recortar(&mut ctx(&w, &r), &[txt("  ho la  ")]).unwrap();
        assert_eq!(result, txt("ho la"));
    }

    #[test]
    fn recortar_ya_limpio_retorna_igual() {
        let (w, r) = null_io();
        let result = recortar(&mut ctx(&w, &r), &[txt("hola")]).unwrap();
        assert_eq!(result, txt("hola"));
    }

    #[test]
    fn contiene_subcadena_presente() {
        let (w, r) = null_io();
        let result = contiene(&mut ctx(&w, &r), &[txt("hola wn++"), txt("wn")]).unwrap();
        assert_eq!(result, Value::Booleano(true));
    }

    #[test]
    fn contiene_subcadena_ausente() {
        let (w, r) = null_io();
        let result = contiene(&mut ctx(&w, &r), &[txt("hola"), txt("mundo")]).unwrap();
        assert_eq!(result, Value::Booleano(false));
    }

    #[test]
    fn empieza_con_correcto() {
        let (w, r) = null_io();
        let si = empieza_con(&mut ctx(&w, &r), &[txt("hola wn"), txt("hola")]).unwrap();
        let no = empieza_con(&mut ctx(&w, &r), &[txt("hola wn"), txt("wn")]).unwrap();
        assert_eq!(si, Value::Booleano(true));
        assert_eq!(no, Value::Booleano(false));
    }

    #[test]
    fn termina_con_correcto() {
        let (w, r) = null_io();
        let si = termina_con(&mut ctx(&w, &r), &[txt("hola wn"), txt("wn")]).unwrap();
        let no = termina_con(&mut ctx(&w, &r), &[txt("hola wn"), txt("hola")]).unwrap();
        assert_eq!(si, Value::Booleano(true));
        assert_eq!(no, Value::Booleano(false));
    }

    #[test]
    fn mayusculas_ascii() {
        let (w, r) = null_io();
        let result = mayusculas(&mut ctx(&w, &r), &[txt("hola wn")]).unwrap();
        assert_eq!(result, txt("HOLA WN"));
    }

    #[test]
    fn minusculas_ascii() {
        let (w, r) = null_io();
        let result = minusculas(&mut ctx(&w, &r), &[txt("HOLA WN")]).unwrap();
        assert_eq!(result, txt("hola wn"));
    }

    #[test]
    fn mayusculas_unicode() {
        // ñ → Ñ (Unicode case mapping)
        let (w, r) = null_io();
        let result = mayusculas(&mut ctx(&w, &r), &[txt("ñoño")]).unwrap();
        assert_eq!(result, txt("ÑOÑO"));
    }

    #[test]
    fn reemplazar_todas_las_ocurrencias() {
        let (w, r) = null_io();
        let result = reemplazar(
            &mut ctx(&w, &r),
            &[txt("aaa bbb aaa"), txt("aaa"), txt("ccc")],
        )
        .unwrap();
        assert_eq!(result, txt("ccc bbb ccc"));
    }

    #[test]
    fn reemplazar_sin_ocurrencias_retorna_igual() {
        let (w, r) = null_io();
        let result = reemplazar(&mut ctx(&w, &r), &[txt("hola"), txt("z"), txt("x")]).unwrap();
        assert_eq!(result, txt("hola"));
    }

    #[test]
    fn repetir_varias_veces() {
        let (w, r) = null_io();
        let result = repetir(&mut ctx(&w, &r), &[txt("ja"), Value::Numero(3.0)]).unwrap();
        assert_eq!(result, txt("jajaja"));
    }

    #[test]
    fn repetir_cero_veces_da_vacio() {
        let (w, r) = null_io();
        let result = repetir(&mut ctx(&w, &r), &[txt("hola"), Value::Numero(0.0)]).unwrap();
        assert_eq!(result, txt(""));
    }

    #[test]
    fn repetir_con_numero_negativo_da_error() {
        let (w, r) = null_io();
        let err = repetir(&mut ctx(&w, &r), &[txt("hola"), Value::Numero(-1.0)]).unwrap_err();
        assert!(matches!(err, VmError::TipoInvalido(_)));
    }

    #[test]
    fn indice_de_encuentra_subcadena() {
        let (w, r) = null_io();
        let result = indice_de(&mut ctx(&w, &r), &[txt("hola mundo"), txt("mundo")]).unwrap();
        assert_eq!(result, Value::Numero(5.0));
    }

    #[test]
    fn indice_de_retorna_nada_si_no_existe() {
        let (w, r) = null_io();
        let result = indice_de(&mut ctx(&w, &r), &[txt("hola"), txt("xyz")]).unwrap();
        assert_eq!(result, Value::Nada);
    }

    #[test]
    fn indice_de_en_unicode_cuenta_chars() {
        // "ñoño" — "o" aparece en posición 1 (índice de char, no de byte)
        let (w, r) = null_io();
        let result = indice_de(&mut ctx(&w, &r), &[txt("ñoño"), txt("o")]).unwrap();
        assert_eq!(result, Value::Numero(1.0));
    }
}
