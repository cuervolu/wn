//! Motor de bytecode para WN++.
//!
//! Pipeline: AST → [`compiler::Compiler`] → [`chunk::Chunk`] → [`vm::VM`]

mod builtins;
pub mod chunk;
pub mod compiler;
mod native;
pub mod opcode;
pub mod value;
pub mod vm;
