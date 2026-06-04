use crate::{value::Value, vm::VmError};
use std::{
    cell::RefCell,
    fmt,
    io::{BufRead, Write},
};

/// Acceso restringido al I/O del VM que se pasa a cada función nativa.
/// Las nativas no tocan el stack ni el GC directamente, eso lo maneja el VM.
///
/// Análogo a `lua_State*` en Lua o `WrenVM*` en Wren.
pub struct NativeContext<'a> {
    pub salida: &'a RefCell<Box<dyn Write>>,
    pub entrada: &'a RefCell<Box<dyn BufRead>>,
}

/// Firma de toda función nativa.
pub type NativeFnPtr = fn(&mut NativeContext, &[Value]) -> Result<Value, VmError>;

/// Descriptor de una función nativa.
///
/// Es `Copy` porque solo tiene punteros y primitivos, cero heap.
/// El VM almacena `Value::Nativa(NativeFn)` directamente en el stack,
/// igual que antes, sin cambio de layout.
#[derive(Clone, Copy)]
pub struct NativeFn {
    pub nombre: &'static str,
    /// `None` = aridad variable (ej: `lorea` acepta N args)
    pub aridad: Option<usize>,
    pub func: NativeFnPtr,
}

impl fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<nativa:{}>", self.nombre)
    }
}

impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool {
        self.nombre == other.nombre
    }
}
