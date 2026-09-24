//! check claude's program generator for CyborgPL (design/verification-session.md).
//! `seed → program` is a pure function: any finding replays from its seed.

pub mod ast;
pub mod gen;
pub mod intervals;
pub mod printer;
pub mod rng;
pub mod shrink;
pub mod solve;

pub use ast::Program;
pub use gen::{generate, Config};
