//! Cyborg Chassis Assembler — the CyborgPL compiler.
//!
//! Pipeline (`design/precog.md`): `.cpls` → tokens → AST → checked AST →
//! MIR → Precog (the last MIR pass) → LLVM IR → binary.

pub mod ast;
pub mod diag;
pub mod concise;
pub mod cyjet_toml;
pub mod format;
pub mod lexer;
pub mod uses;
pub mod names;
pub mod parser;
pub mod argwords;
pub mod cheader;
pub mod template;
pub mod spellings;
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

pub mod precog;
pub mod par;

pub mod codegen;
pub mod report;
