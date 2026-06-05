//! Módulo `lista` de la biblioteca estándar.
//!
//! Funciones para manipulación de listas. Se acceden desde WN++ como:
//! ```text
//! queri lista
//! lista::agregar(xs, 42)
//! lista::ordenar([3, 1, 2])   // → [1, 2, 3]
//! ```
//!
//! ## Mutación vs. valor nuevo
//!
//! Las funciones que modifican la lista original (`agregar`, `quitar`,
//! `insertar`, `eliminar`) retornan `nada` o el elemento removido.
//! Las funciones que construyen una nueva lista (`invertir`, `ordenar`,
//! `slice`, `aplanar`) dejan la original intacta.

use std::{cell::RefCell, rc::Rc};
use wn_vm::{
    native::{NativeContext, NativeFn},
    value::Value,
    vm::VmError,
};

/// Todas las funciones del módulo `lista`.
pub static LISTA: &[NativeFn] = &[
    NativeFn {
        nombre: "agregar",
        aridad: Some(2),
        func: agregar,
    },
    NativeFn {
        nombre: "quitar",
        aridad: Some(1),
        func: quitar,
    },
    NativeFn {
        nombre: "insertar",
        aridad: Some(3),
        func: insertar,
    },
    NativeFn {
        nombre: "eliminar",
        aridad: Some(2),
        func: eliminar,
    },
    NativeFn {
        nombre: "contiene",
        aridad: Some(2),
        func: contiene,
    },
    NativeFn {
        nombre: "invertir",
        aridad: Some(1),
        func: invertir,
    },
    NativeFn {
        nombre: "ordenar",
        aridad: Some(1),
        func: ordenar,
    },
    NativeFn {
        nombre: "slice",
        aridad: Some(3),
        func: slice,
    },
    NativeFn {
        nombre: "primero",
        aridad: Some(1),
        func: primero,
    },
    NativeFn {
        nombre: "ultimo",
        aridad: Some(1),
        func: ultimo,
    },
    NativeFn {
        nombre: "aplanar",
        aridad: Some(1),
        func: aplanar,
    },
];

/// `agregar(lista, valor) → nada` — agrega `valor` al final. Muta la lista.
fn agregar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => {
            xs.borrow_mut().push(args[1].clone());
            Ok(Value::Nada)
        }
        other => Err(esperar_lista("agregar", other)),
    }
}

/// `quitar(lista) → valor` — elimina y retorna el último elemento. Muta la lista.
///
/// Error si la lista está vacía.
fn quitar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => xs.borrow_mut().pop().ok_or_else(|| {
            VmError::TipoInvalido(
                "quitar() no puede sacar elementos de una lista vacía.".to_string(),
            )
        }),
        other => Err(esperar_lista("quitar", other)),
    }
}

/// `insertar(lista, indice, valor) → nada` — inserta `valor` en la posición `indice`. Muta la lista.
/// Acepta índices negativos (se cuentan desde el final).
fn insertar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match (&args[0], &args[1]) {
        (Value::Lista(xs), Value::Numero(n)) => {
            let mut items = xs.borrow_mut();
            let idx = normalizar_indice(*n, items.len(), "insertar")?;
            // insertar en idx == len es válido (equivale a agregar al final)
            let idx = idx.min(items.len());
            items.insert(idx, args[2].clone());
            Ok(Value::Nada)
        }
        (a, b) => Err(VmError::TipoInvalido(format!(
            "insertar() espera (lista, numero, valor), no ({}, {}, ...).",
            a.tipo_nombre(),
            b.tipo_nombre()
        ))),
    }
}

/// `eliminar(lista, indice) → valor` — elimina y retorna el elemento en `indice`. Muta la lista.
///
/// Acepta índices negativos.
fn eliminar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match (&args[0], &args[1]) {
        (Value::Lista(xs), Value::Numero(n)) => {
            let mut items = xs.borrow_mut();
            let idx = normalizar_indice(*n, items.len(), "eliminar")?;
            if idx >= items.len() {
                return Err(VmError::IndiceInvalido {
                    indice: idx as i64,
                    largo: items.len(),
                });
            }
            Ok(items.remove(idx))
        }
        (a, b) => Err(VmError::TipoInvalido(format!(
            "eliminar() espera (lista, numero), no ({}, {}).",
            a.tipo_nombre(),
            b.tipo_nombre()
        ))),
    }
}

/// `contiene(lista, valor) → booleano` — indica si `valor` está en la lista.
///
/// Usa igualdad estructural (mismo tipo y valor).
fn contiene(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => Ok(Value::Booleano(xs.borrow().contains(&args[1]))),
        other => Err(esperar_lista("contiene", other)),
    }
}

/// `invertir(lista) → lista` — retorna una nueva lista con los elementos en orden inverso.
///
/// La lista original no se modifica.
fn invertir(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => {
            let mut items = xs.borrow().clone();
            items.reverse();
            Ok(Value::Lista(Rc::new(RefCell::new(items))))
        }
        other => Err(esperar_lista("invertir", other)),
    }
}

/// `ordenar(lista) → lista` — retorna una nueva lista ordenada.
///
/// Solo funciona con listas homogéneas de `numero` o `texto`.
/// Error si la lista mezcla tipos.
/// La lista original no se modifica.
fn ordenar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => {
            let mut items = xs.borrow().clone();
            let mut err: Option<VmError> = None;
            items.sort_by(|a, b| {
                ordenar_cmp(a, b).unwrap_or_else(|e| {
                    err.get_or_insert(e);
                    std::cmp::Ordering::Equal
                })
            });
            if let Some(e) = err {
                return Err(e);
            }
            Ok(Value::Lista(Rc::new(RefCell::new(items))))
        }
        other => Err(esperar_lista("ordenar", other)),
    }
}

/// `slice(lista, inicio, fin) → lista` — retorna los elementos entre `inicio` (inclusivo)
/// y `fin` (exclusivo).
///
/// Acepta índices negativos al estilo Python: `slice(xs, -2, largo(xs))` retorna los
/// últimos dos elementos.
/// La lista original no se modifica.
fn slice(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match (&args[0], &args[1], &args[2]) {
        (Value::Lista(xs), Value::Numero(inicio), Value::Numero(fin)) => {
            let items = xs.borrow();
            let len = items.len();
            let start = normalizar_indice(*inicio, len, "slice")?.min(len);
            let end = normalizar_indice(*fin, len, "slice")?.min(len);
            let end = end.max(start); // slice vacío si fin < inicio
            Ok(Value::Lista(Rc::new(RefCell::new(
                items[start..end].to_vec(),
            ))))
        }
        (a, b, c) => Err(VmError::TipoInvalido(format!(
            "slice() espera (lista, numero, numero), no ({}, {}, {}).",
            a.tipo_nombre(),
            b.tipo_nombre(),
            c.tipo_nombre()
        ))),
    }
}

/// `primero(lista) → valor | nada` — retorna el primer elemento, o `nada` si la lista está vacía.
fn primero(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => Ok(xs.borrow().first().cloned().unwrap_or(Value::Nada)),
        other => Err(esperar_lista("primero", other)),
    }
}

/// `ultimo(lista) → valor | nada` — retorna el último elemento, o `nada` si la lista está vacía.
fn ultimo(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => Ok(xs.borrow().last().cloned().unwrap_or(Value::Nada)),
        other => Err(esperar_lista("ultimo", other)),
    }
}

/// `aplanar(lista) → lista` — aplana un nivel de anidamiento.
///
/// `aplanar([[1, 2], [3, 4]])` → `[1, 2, 3, 4]`
/// Los elementos que no son listas se incluyen tal cual.
fn aplanar(_ctx: &mut NativeContext, args: &[Value]) -> Result<Value, VmError> {
    match &args[0] {
        Value::Lista(xs) => {
            let mut resultado = Vec::new();
            for item in xs.borrow().iter() {
                match item {
                    Value::Lista(sub) => resultado.extend(sub.borrow().iter().cloned()),
                    otro => resultado.push(otro.clone()),
                }
            }
            Ok(Value::Lista(Rc::new(RefCell::new(resultado))))
        }
        other => Err(esperar_lista("aplanar", other)),
    }
}

/// Convierte un índice f64 (puede ser negativo) a un `usize` dentro del rango.
fn normalizar_indice(n: f64, len: usize, nombre: &str) -> Result<usize, VmError> {
    if !n.is_finite() || n.fract() != 0.0 {
        return Err(VmError::TipoInvalido(format!(
            "{nombre}() espera un índice entero, no {n}."
        )));
    }
    let i = n as i64;
    if i >= 0 {
        Ok(i as usize)
    } else {
        let abs = (-i) as usize;
        Ok(len.saturating_sub(abs))
    }
}

fn ordenar_cmp(a: &Value, b: &Value) -> Result<std::cmp::Ordering, VmError> {
    match (a, b) {
        (Value::Numero(x), Value::Numero(y)) => {
            Ok(x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
        }
        (Value::Texto(x), Value::Texto(y)) => Ok(x.cmp(y)),
        _ => Err(VmError::TipoInvalido(
            "ordenar() solo funciona con listas homogéneas de numero o texto.".to_string(),
        )),
    }
}

fn esperar_lista(nombre: &str, recibido: &Value) -> VmError {
    VmError::TipoInvalido(format!(
        "{nombre}() espera una lista, no '{}'.",
        recibido.tipo_nombre()
    ))
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

    fn num(n: f64) -> Value {
        Value::Numero(n)
    }

    fn txt(s: &str) -> Value {
        Value::Texto(Rc::from(s))
    }

    fn lista(xs: Vec<Value>) -> Value {
        Value::Lista(Rc::new(RefCell::new(xs)))
    }

    fn extraer(v: Value) -> Vec<Value> {
        match v {
            Value::Lista(xs) => xs.borrow().clone(),
            other => panic!("se esperaba lista, se obtuvo {other:?}"),
        }
    }

    #[test]
    fn lista_registra_todas_las_funciones() {
        let nombres: Vec<_> = LISTA.iter().map(|f| f.nombre).collect();
        for esperado in [
            "agregar", "quitar", "insertar", "eliminar", "contiene", "invertir", "ordenar",
            "slice", "primero", "ultimo", "aplanar",
        ] {
            assert!(nombres.contains(&esperado), "falta '{esperado}' en LISTA");
        }
    }

    #[test]
    fn agregar_muta_la_lista_original() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0)]);
        agregar(&mut ctx(&w, &r), &[xs.clone(), num(3.0)]).unwrap();
        assert_eq!(extraer(xs), vec![num(1.0), num(2.0), num(3.0)]);
    }

    #[test]
    fn agregar_retorna_nada() {
        let (w, r) = null_io();
        let xs = lista(vec![]);
        let result = agregar(&mut ctx(&w, &r), &[xs, num(1.0)]).unwrap();
        assert_eq!(result, Value::Nada);
    }

    #[test]
    fn quitar_retorna_ultimo_y_muta() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0), num(3.0)]);
        let result = quitar(&mut ctx(&w, &r), std::slice::from_ref(&xs)).unwrap();
        assert_eq!(result, num(3.0));
        assert_eq!(extraer(xs), vec![num(1.0), num(2.0)]);
    }

    #[test]
    fn quitar_lista_vacia_da_error() {
        let (w, r) = null_io();
        let xs = lista(vec![]);
        let err = quitar(&mut ctx(&w, &r), &[xs]).unwrap_err();
        assert!(matches!(err, VmError::TipoInvalido(_)));
    }

    #[test]
    fn insertar_al_inicio() {
        let (w, r) = null_io();
        let xs = lista(vec![num(2.0), num(3.0)]);
        insertar(&mut ctx(&w, &r), &[xs.clone(), num(0.0), num(1.0)]).unwrap();
        assert_eq!(extraer(xs), vec![num(1.0), num(2.0), num(3.0)]);
    }

    #[test]
    fn insertar_indice_negativo() {
        let (w, r) = null_io();
        // insertar en -1 de [1, 3] → [1, 2, 3]
        let xs = lista(vec![num(1.0), num(3.0)]);
        insertar(&mut ctx(&w, &r), &[xs.clone(), num(-1.0), num(2.0)]).unwrap();
        assert_eq!(extraer(xs), vec![num(1.0), num(2.0), num(3.0)]);
    }

    #[test]
    fn eliminar_retorna_elemento_y_muta() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0), num(3.0)]);
        let result = eliminar(&mut ctx(&w, &r), &[xs.clone(), num(1.0)]).unwrap();
        assert_eq!(result, num(2.0));
        assert_eq!(extraer(xs), vec![num(1.0), num(3.0)]);
    }

    #[test]
    fn eliminar_indice_fuera_de_rango_da_error() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0)]);
        let err = eliminar(&mut ctx(&w, &r), &[xs, num(5.0)]).unwrap_err();
        assert!(matches!(err, VmError::IndiceInvalido { .. }));
    }

    #[test]
    fn contiene_elemento_presente() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0), num(3.0)]);
        let result = contiene(&mut ctx(&w, &r), &[xs, num(2.0)]).unwrap();
        assert_eq!(result, Value::Booleano(true));
    }

    #[test]
    fn contiene_elemento_ausente() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0)]);
        let result = contiene(&mut ctx(&w, &r), &[xs, num(9.0)]).unwrap();
        assert_eq!(result, Value::Booleano(false));
    }

    #[test]
    fn invertir_no_muta_original() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0), num(3.0)]);
        let inv = invertir(&mut ctx(&w, &r), std::slice::from_ref(&xs)).unwrap();
        // original sin cambios
        assert_eq!(extraer(xs), vec![num(1.0), num(2.0), num(3.0)]);
        // resultado invertido
        assert_eq!(extraer(inv), vec![num(3.0), num(2.0), num(1.0)]);
    }

    #[test]
    fn invertir_lista_vacia() {
        let (w, r) = null_io();
        let result = invertir(&mut ctx(&w, &r), &[lista(vec![])]).unwrap();
        assert_eq!(extraer(result), vec![]);
    }

    #[test]
    fn ordenar_numeros() {
        let (w, r) = null_io();
        let xs = lista(vec![num(3.0), num(1.0), num(2.0)]);
        let result = ordenar(&mut ctx(&w, &r), &[xs]).unwrap();
        assert_eq!(extraer(result), vec![num(1.0), num(2.0), num(3.0)]);
    }

    #[test]
    fn ordenar_textos_lexicografico() {
        let (w, r) = null_io();
        let xs = lista(vec![txt("banana"), txt("apple"), txt("cereza")]);
        let result = ordenar(&mut ctx(&w, &r), &[xs]).unwrap();
        assert_eq!(
            extraer(result),
            vec![txt("apple"), txt("banana"), txt("cereza")]
        );
    }

    #[test]
    fn ordenar_no_muta_original() {
        let (w, r) = null_io();
        let xs = lista(vec![num(3.0), num(1.0)]);
        ordenar(&mut ctx(&w, &r), std::slice::from_ref(&xs)).unwrap();
        assert_eq!(extraer(xs), vec![num(3.0), num(1.0)]);
    }

    #[test]
    fn ordenar_tipos_mixtos_da_error() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), txt("hola")]);
        let err = ordenar(&mut ctx(&w, &r), &[xs]).unwrap_err();
        assert!(matches!(err, VmError::TipoInvalido(_)));
    }

    #[test]
    fn slice_rango_normal() {
        let (w, r) = null_io();
        let xs = lista(vec![num(0.0), num(1.0), num(2.0), num(3.0)]);
        let result = slice(&mut ctx(&w, &r), &[xs, num(1.0), num(3.0)]).unwrap();
        assert_eq!(extraer(result), vec![num(1.0), num(2.0)]);
    }

    #[test]
    fn slice_indice_negativo() {
        let (w, r) = null_io();
        // slice(xs, -2, 4) → últimos 2 elementos de [0,1,2,3]
        let xs = lista(vec![num(0.0), num(1.0), num(2.0), num(3.0)]);
        let result = slice(&mut ctx(&w, &r), &[xs, num(-2.0), num(4.0)]).unwrap();
        assert_eq!(extraer(result), vec![num(2.0), num(3.0)]);
    }

    #[test]
    fn slice_fin_menor_que_inicio_da_lista_vacia() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), num(2.0)]);
        let result = slice(&mut ctx(&w, &r), &[xs, num(2.0), num(0.0)]).unwrap();
        assert_eq!(extraer(result), vec![]);
    }

    #[test]
    fn primero_retorna_primer_elemento() {
        let (w, r) = null_io();
        let xs = lista(vec![num(10.0), num(20.0)]);
        assert_eq!(primero(&mut ctx(&w, &r), &[xs]).unwrap(), num(10.0));
    }

    #[test]
    fn primero_lista_vacia_retorna_nada() {
        let (w, r) = null_io();
        assert_eq!(
            primero(&mut ctx(&w, &r), &[lista(vec![])]).unwrap(),
            Value::Nada
        );
    }

    #[test]
    fn ultimo_retorna_ultimo_elemento() {
        let (w, r) = null_io();
        let xs = lista(vec![num(10.0), num(20.0)]);
        assert_eq!(ultimo(&mut ctx(&w, &r), &[xs]).unwrap(), num(20.0));
    }

    #[test]
    fn ultimo_lista_vacia_retorna_nada() {
        let (w, r) = null_io();
        assert_eq!(
            ultimo(&mut ctx(&w, &r), &[lista(vec![])]).unwrap(),
            Value::Nada
        );
    }

    #[test]
    fn aplanar_un_nivel() {
        let (w, r) = null_io();
        let xs = lista(vec![
            lista(vec![num(1.0), num(2.0)]),
            lista(vec![num(3.0), num(4.0)]),
        ]);
        let result = aplanar(&mut ctx(&w, &r), &[xs]).unwrap();
        assert_eq!(
            extraer(result),
            vec![num(1.0), num(2.0), num(3.0), num(4.0)]
        );
    }

    #[test]
    fn aplanar_elementos_no_lista_se_incluyen_tal_cual() {
        let (w, r) = null_io();
        let xs = lista(vec![num(1.0), lista(vec![num(2.0), num(3.0)])]);
        let result = aplanar(&mut ctx(&w, &r), &[xs]).unwrap();
        assert_eq!(extraer(result), vec![num(1.0), num(2.0), num(3.0)]);
    }

    #[test]
    fn aplanar_solo_un_nivel_profundidad() {
        let (w, r) = null_io();
        // [[1, [2]]] → [1, [2]]  (no aplana el [2] adentro)
        let xs = lista(vec![lista(vec![num(1.0), lista(vec![num(2.0)])])]);
        let result = aplanar(&mut ctx(&w, &r), &[xs]).unwrap();
        // El [2] sigue siendo lista después de un nivel
        let partes = extraer(result);
        assert_eq!(partes[0], num(1.0));
        assert!(matches!(partes[1], Value::Lista(_)));
    }
}
