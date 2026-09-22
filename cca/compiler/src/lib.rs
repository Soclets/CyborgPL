//! Cyborg Chassis Assembler — the CyborgPL compiler.
//!
//! Pipeline (`design/precog.md`): `.cpls` → tokens → AST → checked AST →
//! MIR → Precog (the last MIR pass) → LLVM IR → binary.

pub mod ast;
pub mod diag;
pub mod lexer;
pub mod names;
pub mod parser;
pub mod source;
