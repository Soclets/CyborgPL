//! The base (`design/cores.md`): it reads files, loads the codextra and
//! the cores, enforces their grants, and hands the codextra's LLVM IR to
//! LLVM unread. Filled step by step by `cca/PLUGINS.md`'s build order; the
//! power ledger (step 2) is the first part.

pub mod powers;
