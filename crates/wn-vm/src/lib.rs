//! Motor de bytecode para WN++.
//!
//! Pipeline: AST  [`compiler::Compiler`]  [`chunk::Chunk`]  [`vm::VM`]

pub mod builtins;
pub mod chunk;
pub mod compiler;
pub mod native;
pub mod opcode;
pub mod resolver;
pub mod value;
pub mod vm;
