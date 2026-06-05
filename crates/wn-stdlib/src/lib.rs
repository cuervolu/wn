//! Biblioteca estándar de WN++.
//!
//! Organizada en módulos por dominio. Cada módulo expone un slice estático
//! de [`NativeFn`] que el sistema de módulos registra bajo demanda via `queri`.
//!
//! ```text
//! queri texto
//! queri lista
//! texto::dividir("hola wn", " ")
//! ```

pub mod lista;
pub mod stdlib_resolver;
pub mod cadena;
