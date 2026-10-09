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
pub mod check;
pub mod hir;
pub mod resolve;
pub mod types;
pub mod rounding;
pub mod bigint;
pub mod deci;
pub mod load;
pub mod target;

pub mod mir;

pub mod cortex;
pub mod par;

pub mod codegen;
