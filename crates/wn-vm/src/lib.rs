//! Motor de bytecode para WN++.
//!
//! Pipeline: AST → [`compiler::Compiler`] → [`chunk::Chunk`] → [`vm::VM`]

pub mod chunk;
pub mod compiler;
pub mod opcode;
pub mod value;
pub mod vm;
