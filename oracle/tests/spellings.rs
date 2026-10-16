//! `spellings.cyr`, the one table of the language's words (owner, 2026-10-13), checked against itself and the
//! naming rule: no hint names an old spelling, nothing is both current and old, every keyword lowercase
//! with `_` unless it is a capital the table allows.

use std::process::Command;

#[test]
fn spellings_table_is_consistent() {
    let out = Command::new(env!("CARGO_BIN_EXE_oracle")).arg("--check-spellings").output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}
