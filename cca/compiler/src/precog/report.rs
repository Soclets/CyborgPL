//! The build's report (`precog.md`): a line per check site, in source
//! order, and the three lists `core-features.md` asks for.

use std::fmt::Write as _;

use crate::ast::Tier;
use crate::mir::{Program, Verdict};
use crate::source::SourceMap;

use super::Analysis;

pub fn render(a: &Analysis, p: &Program, sources: &SourceMap) -> String {
    let mut lines: Vec<(u32, u32, String)> = Vec::new();
    for r in &a.sites {
        let f = &p.functions[r.function];
        let site = f.site(r.site);
        let span = f.span(site.span);
        let file = sources.get(span.file);
        let (line, col) = file.line_col(span.start);
        let code = file.slice(span).lines().next().unwrap_or("").trim().to_string();
        let word = match &r.verdict {
            Verdict::Proven { .. } => "proven",
            Verdict::Fails { .. } => match site.tier {
                Some(Tier::Check) => "refused",
                _ => "checked",
            },
            Verdict::Trusted => "trusted",
            Verdict::Unproven => "checked",
        };
        let what = crate::types::TySpelling { ty: &crate::types::Ty::Nothing, structs: &p.items.structs, enums: &p.items.enums };
        let _ = what;
        let kind = site.kind.sentence(&|t| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string());
        lines.push((span.file.0, line, format!("line {:<4} {:<34} {:<9} ({})", line, truncate(&code, 34), word, if r.note.is_empty() { kind } else { r.note.clone() })));
        let _ = col;
    }
    lines.sort_by_key(|(f, l, _)| (*f, *l));
    let mut o = String::new();
    if a.whole_program {
        let _ = writeln!(o, "The program reads nothing from outside, so Precog ran all of it ({} steps).", a.steps);
    } else {
        let _ = writeln!(o, "Precog ran the program over what its inputs allow ({} steps).", a.steps);
    }
    for (_, _, l) in &lines {
        let _ = writeln!(o, "{}", l);
    }
    let proven = a.sites.iter().filter(|r| matches!(r.verdict, Verdict::Proven { .. })).count();
    let trusted = a.sites.iter().filter(|r| matches!(r.verdict, Verdict::Trusted)).count();
    let checked = a.sites.len() - proven - trusted;
    let _ = writeln!(o, "{} site(s): {} proven, {} checked at run time, {} trusted", a.sites.len(), proven, checked, trusted);
    if !a.portfolio.is_empty() {
        let _ = writeln!(o, "solvers: {}", a.portfolio);
    }
    if a.solved > 0 {
        let _ = writeln!(o, "{} site(s) layer 2 could not settle were asked exactly, and answered", a.solved);
    }
    for d in &a.disagreements {
        let _ = writeln!(o, "\nnote:  {}", d);
    }
    if a.folded > 0 {
        let _ = writeln!(o, "{} instruction(s) were worked out at compile time and are not in the binary", a.folded);
    }
    o
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let t: String = s.chars().take(n - 1).collect();
        format!("{}…", t)
    }
}
