//! Redesign three (2026-10-28; oracle c9f4f01b), until the generator writes it
//! itself: a program in today's syntax taken there by design claude's
//! converters, `oracle --redesign-show`, then `oracle/tools/redesign_three.py`
//! and `plain_verbs.py` (which runs `oracle/target/debug/oracle`). Under
//! `CHECK_THREE=1` every program the comparison hands the engines goes this way
//! first, so kayo reading only redesign three can be compared meanwhile; `check
//! redesign` checks the conversion itself against an oracle from before it.
//!
//! The converter is the oracle's author's, so a mistake in it reaches both
//! engines alike: it can hide nothing between them, only change what is
//! compared, and `check redesign` keeps that honest.

use crate::engines;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

/// Whether `CHECK_THREE=1`.
pub fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHECK_THREE").is_ok_and(|v| v == "1"))
}

/// The program in redesign three, or why the converters stopped. `dir` gets
/// the files.
pub fn convert(text: &str, dir: &Path) -> Result<String, String> {
    let repo = crate::repo();
    let oracle = repo.join("oracle/target/release/oracle");
    let tools = repo.join("oracle/tools");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let (f, g) = (dir.join("today.cy"), dir.join("three.cy"));
    std::fs::write(&f, text).map_err(|e| e.to_string())?;
    let c = engines::run(Command::new(&oracle).arg("--redesign-show").arg(&f), dir, &[], false, engines::RUN_LIMIT);
    if c.code != Some(0) || c.stdout.is_empty() {
        return Err(format!("oracle --redesign-show: {}", c.stderr.lines().next().unwrap_or("").chars().take(160).collect::<String>()));
    }
    // the converter guesses `fn` / `let` when it cannot read the old program
    // through (design claude, 2f7fd026): a stop, not a conversion
    if let Some(l) = c.stderr.lines().find(|l| l.contains("not analysed")) {
        return Err(format!("oracle --redesign-show: {}", l.chars().take(160).collect::<String>()));
    }
    std::fs::write(&g, &c.stdout).map_err(|e| e.to_string())?;
    for tool in ["redesign_three.py", "plain_verbs.py"] {
        let t = engines::run(Command::new("python3").arg(tools.join(tool)).arg(&g), dir, &[], false, engines::RUN_LIMIT);
        if t.code != Some(0) {
            return Err(format!("{tool}: {}", t.stderr.lines().last().unwrap_or("").chars().take(160).collect::<String>()));
        }
    }
    let three = std::fs::read_to_string(&g).map_err(|e| e.to_string())?;
    Ok(with_sigs(&three))
}

/// The generics episode's `[<T>: any]` (e0ff1255) adds with `+`, which since 2550064a comes
/// only from a `sig` in the bound: such a function is bound by `Addable`, as
/// examples/generic_literal.cy writes it. The generator's own hand edit, which
/// no converter can make.
pub fn with_sigs(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 96);
    let mut needed = false;
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i];
        if let Some(name) = l.strip_prefix("fn ").and_then(|r| r.split_whitespace().next()).filter(|_| l.contains("[<T>: any]")) {
            let close = format!(";fn {name}");
            let end = (i..lines.len()).find(|&k| lines[k].trim() == close).unwrap_or(i);
            if lines[i + 1..end].iter().any(|b| b.contains(" + ")) {
                needed = true;
                out.push_str(&l.replace("[<T>: any]", "[<T>: Addable]"));
                out.push('\n');
                i += 1;
                continue;
            }
        }
        out.push_str(l);
        out.push('\n');
        i += 1;
    }
    if !needed {
        return out;
    }
    // after the `use` lines, which go first
    let head: usize = out.lines().take_while(|l| l.starts_with("use ") || l.starts_with('#') || l.trim().is_empty()).map(|l| l.len() + 1).sum();
    format!("{}sig Addable [<T>]:\n    op + [lent $a: <T>, lent $b: <T>] -> <T>;\n;sig Addable\n\n{}", &out[..head], &out[head..])
}

#[cfg(test)]
mod tests {
    use super::with_sigs;

    #[test]
    fn addable_where_a_generic_adds() {
        let s = "use std::ls as ls;\nfn g1 [<T>: any] [lent $xs: std::ls[<T>]] -> <T>:\n    change $s to $s + $x;\n;fn g1\nfn g2 [<T>: any] [let $x: <T>] -> <T>:\n    ret $x;\n;fn g2\n";
        assert_eq!(with_sigs(s), "use std::ls as ls;\nsig Addable [<T>]:\n    op + [lent $a: <T>, lent $b: <T>] -> <T>;\n;sig Addable\n\nfn g1 [<T>: Addable] [lent $xs: std::ls[<T>]] -> <T>:\n    change $s to $s + $x;\n;fn g1\nfn g2 [<T>: any] [let $x: <T>] -> <T>:\n    ret $x;\n;fn g2\n");
    }
}
