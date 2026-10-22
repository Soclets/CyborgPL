//! The build report (owner, 2026-10-19; `design/tooling.md` § The build
//! report): everything a build did, in a Markdown file beside the program,
//! `build/fib.report.md` for `-o build/fib`. One heading per section, a
//! contents line at the top, and as much detail as the build knows; the
//! terminal keeps errors, warnings and one summary line.

use std::collections::HashSet;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::ast::Tier;
use crate::codegen::size::{bytes_text, Sizes};
use crate::codegen::{ObjectFacts, StructLayout};
use crate::mir::{Const, Function, Operand, Program, Rvalue, SiteKind, StmtKind, Terminator, Verdict};
use crate::precog::Analysis;
use crate::source::{SourceMap, Span};
use crate::types::{Ty, TySpelling};

/// What one build knew, handed over at its end.
pub struct Facts<'a> {
    pub source: &'a Path,
    pub program: &'a Path,
    pub final_build: bool,
    pub assert_proofs: bool,
    pub started: SystemTime,
    pub total: Duration,
    /// Each compile phase, in order, and how long it took.
    pub phases: &'a [(&'static str, Duration)],
    pub mir: &'a Program,
    pub sources: &'a SourceMap,
    pub analysis: &'a Analysis,
    pub plan: &'a crate::par::Plan,
    pub sizes: Result<&'a Sizes, &'a str>,
    pub structs: &'a [StructLayout],
    pub consts: &'a [(String, String, u64)],
    pub object: &'a ObjectFacts,
    /// Precog's notes, rendered as the terminal shows them.
    pub notes: &'a [String],
    /// A final build's debug info, where it went.
    pub debug: Option<&'a Path>,
    /// The `incl.C` lines read, and how many declarations each brought.
    pub c_lines: &'a [(String, usize)],
    pub tests: bool,
}

/// Where a program's report goes: beside it, named after it.
pub fn path_for(program: &Path) -> PathBuf {
    let mut name = program.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(".report.md");
    program.with_file_name(name)
}

const SECTIONS: [&str; 11] = ["Summary", "Inputs", "Proofs", "Your markers", "Types Precog picked", "Optimizations", "Removed", "C", "Sizes", "Timing", "Notes"];

/// The report, as Markdown.
pub fn render(f: &Facts) -> String {
    let p = f.mir;
    let spell = |t: &Ty| -> String {
        let said = TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
        // A bare literal's type is built at 64 bits, as the code generator builds it.
        match t.base() {
            Ty::Int(crate::types::IntW::Free(_)) => format!("{} (built as i64)", said),
            Ty::Bin(crate::types::BinW::Free(_)) => format!("{} (built as f64)", said),
            _ => said,
        }
    };
    let where_ = |s: Span| -> String {
        let file = f.sources.get(s.file);
        let (line, col) = file.line_col(s.start);
        format!("{}:{}:{}", file.display_name(), line, col)
    };
    let code_at = |s: Span| -> String {
        let file = f.sources.get(s.file);
        file.slice(s).lines().next().unwrap_or("").trim().to_string()
    };
    let live = |i: usize| -> bool { !p.functions[i].dropped && (f.tests || !p.functions[i].is_test) };
    let fname = |func: &Function| -> String { if func.name.is_empty() { "MAIN".to_string() } else { func.name.clone() } };

    let mut sec: Vec<(String, usize)> = Vec::new();

    // ---------------------------------------------------------------- Inputs
    let mut inputs = String::new();
    let mut files: Vec<_> = f.sources.files.iter().filter(|s| s.path != Path::new("the base")).collect();
    // A manifest is read more than once (its targets before its code).
    let mut listed = HashSet::new();
    files.retain(|s| listed.insert(s.path.clone()));
    let _ = writeln!(inputs, "**Source files** ({}):\n", files.len());
    let _ = writeln!(inputs, "| File | Lines | Bytes |\n|---|---:|---:|");
    for s in &files {
        let _ = writeln!(inputs, "| {} | {} | {} |", cell(&tidy(&s.path)), s.text.lines().count(), group(s.text.len() as u64));
    }
    let _ = writeln!(inputs, "\nThe base (`optional` and the language's own words) is built into kayo.\n");
    // A concise file's decoded text (owner, 2026-10-22; `design/concise.md`):
    // what was read, since the file itself is only binary.
    for s in files.iter().filter(|s| s.concise) {
        let _ = writeln!(inputs, "**{}** is concise CyborgPL, read as:\n\n```\n{}\n```\n", tidy(&s.path), s.text.trim_end());
    }
    let kits = &p.items.report_kits;
    if kits.iter().any(|k| k.program) {
        let _ = writeln!(inputs, "**Kits** ({}):\n", kits.len());
    } else {
        let _ = writeln!(inputs, "**Kits**: the program is a lone file, with no `CyJet.cyr`: it gets the installed std{}.\n", if kits.is_empty() { ", which is not here" } else { "" });
    }
    if !kits.is_empty() {
        let _ = writeln!(inputs, "| Prefix | Name | Version | What | Folder | Limbs |\n|---|---|---|---|---|---|");
        for k in kits {
            let what = if k.program { "this program" } else if k.std { "std" } else { "a dependency" };
            let limbs = if k.limbs.is_empty() {
                "none".to_string()
            } else {
                k.limbs.iter().map(|l| {
                    let v = crate::ast::OFFICIAL_CORES.iter().find(|(n, _, _)| n == l).map(|(_, _, v)| *v).unwrap_or("");
                    format!("{} {}", l, v)
                }).collect::<Vec<_>>().join(", ")
            };
            let _ = writeln!(inputs, "| `{}` | {} | {} | {} | {} | {} |", k.prefix, cell(k.name.as_deref().unwrap_or("—")), cell(k.version.as_deref().unwrap_or("—")), what, cell(&tidy(&k.dir)), cell(&limbs));
        }
    }
    let _ = writeln!(inputs, "\n**Settings that applied**:\n");
    let _ = writeln!(inputs, "- Target built: `{}`", crate::target::name());
    if let Some(k) = kits.iter().find(|k| k.program) {
        if !k.targets.is_empty() {
            let _ = writeln!(inputs, "- Targets listed: {}", k.targets.iter().map(|t| format!("`{}`{}", t, if k.default_target.as_deref() == Some(t) { " (default)" } else { "" })).collect::<Vec<_>>().join(", "));
        }
        for t in &k.targets {
            match k.table_bytes.iter().find(|(n, _)| n == t) {
                Some((_, b)) => {
                    let _ = writeln!(inputs, "- `table_bytes` on `{}`: {}", t, group(*b));
                }
                None => {
                    let _ = writeln!(inputs, "- `table_bytes` on `{}`: 65,536 (not written)", t);
                }
            }
        }
    } else {
        let _ = writeln!(inputs, "- `table_bytes`: 65,536 (no `CyJet.cyr`)");
    }
    let marked: Vec<String> = p.items.fns.iter().filter_map(|d| d.table_bytes.map(|b| format!("`{}`: {}", d.name, group(b)))).collect();
    if !marked.is_empty() {
        let _ = writeln!(inputs, "- `table_bytes` on functions: {}", marked.join(", "));
    }
    let _ = writeln!(inputs, "- Build: {}{}", if f.final_build { "final (`--final`): every check proven, none left at run time" } else { "working: what Precog could not prove is checked at run time" }, if f.assert_proofs { "; proofs asserted (`--assert-proofs`): every proven site is checked anyway" } else { "" });
    sec.push((inputs, files.len()));

    // ---------------------------------------------------------------- Proofs
    let mut rows: Vec<(String, u32, String)> = Vec::new();
    let (mut proven, mut checked, mut trusted) = (0, 0, 0);
    for r in &f.analysis.sites {
        let func = &p.functions[r.function];
        let site = func.site(r.site);
        let span = func.span(site.span);
        let (verdict, how) = match &r.verdict {
            Verdict::Proven { how } => {
                proven += 1;
                ("proven", if r.note.is_empty() || r.note == *how { how.to_string() } else if r.note.starts_with(how) { r.note.clone() } else { format!("{}: {}", how, r.note) })
            }
            Verdict::Trusted => {
                trusted += 1;
                ("not checked", format!("`{}` on your word: see Your markers", tier_word(site.tier)))
            }
            Verdict::Fails { counterexample } => {
                checked += 1;
                ("checked at run time", format!("can fail: {}", counterexample))
            }
            Verdict::Unproven => {
                checked += 1;
                ("checked at run time", if r.note.is_empty() { "not proven".to_string() } else { r.note.clone() })
            }
        };
        let file = f.sources.get(span.file);
        let (line, _) = file.line_col(span.start);
        rows.push((file.display_name(), line, format!("| {} | {} | {} | {} | {} | {} |", cell(&where_(span)), cell(&fname(func)), kind_name(&site.kind, &spell), code(&code_at(span)), verdict, cell(&how))));
    }
    rows.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    let mut proofs = String::new();
    let _ = writeln!(proofs, "{} check site{}: {} proven, {} checked at run time, {} not checked on your word. {}\n", f.analysis.sites.len(), plural(f.analysis.sites.len()), proven, checked, trusted, if f.analysis.whole_program { format!("The program reads nothing from outside, so Precog ran all of it ({} steps).", group(f.analysis.steps)) } else { format!("Precog ran the program over what its inputs allow ({} steps).", group(f.analysis.steps)) });
    if !f.analysis.portfolio.is_empty() {
        let _ = writeln!(proofs, "Solvers: {}.\n", f.analysis.portfolio);
    }
    if f.analysis.solved > 0 {
        let _ = writeln!(proofs, "The solvers settled {} site{} the ranges could not.\n", f.analysis.solved, plural(f.analysis.solved));
    }
    if !rows.is_empty() {
        let _ = writeln!(proofs, "| Where | In | Kind | Code | Verdict | How |\n|---|---|---|---|---|---|");
        for (_, _, r) in &rows {
            let _ = writeln!(proofs, "{}", r);
        }
    }
    for d in &f.analysis.disagreements {
        let _ = writeln!(proofs, "\n- The solvers disagreed (a bug in a solver or in kayo, never in the program): {}", d);
    }
    sec.push((proofs, f.analysis.sites.len()));

    // ---------------------------------------------------------- Your markers
    let mut markers = String::new();
    let mut n_markers = 0;
    let mut passed = Vec::new();
    let mut unchecked = Vec::new();
    for (fi, func) in p.functions.iter().enumerate() {
        if !live(fi) || func.nocheat_of.is_some() {
            continue;
        }
        for m in &func.markers {
            let span = func.span(m.span);
            // What it covers: its sites, and the calls it makes.
            let mut covers: Vec<String> = (m.sites.0..m.sites.1).filter_map(|si| func.sites.get(si as usize)).map(|site| site.sentence(&spell)).collect();
            for b in m.exits() {
                let Some(bl) = func.blocks.get(b as usize) else { continue };
                if let Terminator::Call { callee, .. } = &bl.term {
                    match callee {
                        crate::mir::Callee::Extern(id) => {
                            let e = &p.items.externs[id.0 as usize];
                            // Only `NO_FUCKING_CHECKS` trusts a ranged answer; under
                            // `no_check` the door still checks it.
                            covers.push(match (e.ranged, m.tier) {
                                (true, Tier::NoChecks) => format!("trusts that C's `{}` answers within `{}`, unchecked", e.name, spell(&e.ret)),
                                (true, _) => format!("vouches for C's `{}` (its answer is still checked to be within `{}`)", e.name, spell(&e.ret)),
                                _ => format!("vouches for C's `{}`", e.name),
                            });
                        }
                        crate::mir::Callee::Fn(g) => covers.push(format!("calls `{}`", fname(&p.functions[g.0 as usize]))),
                        crate::mir::Callee::Value(_) => covers.push("calls a held function".to_string()),
                        _ => {}
                    }
                }
            }
            covers.dedup();
            let what = if covers.is_empty() { "nothing Precog would have checked".to_string() } else { covers.join("; ") };
            let row = (where_(span), tier_word(Some(m.tier)), code_at(span), what);
            match m.tier {
                Tier::Check => passed.push(row),
                _ => unchecked.push(row),
            }
        }
    }
    passed.sort();
    passed.dedup();
    unchecked.sort();
    unchecked.dedup();
    n_markers += passed.len() + unchecked.len() + f.analysis.certain_under_nocheck.len();
    let _ = writeln!(markers, "**`check`** ({}): what each must prove; a build that could not prove one would have been refused.\n", passed.len());
    if !passed.is_empty() {
        let _ = writeln!(markers, "| Where | Statement | What it proved |\n|---|---|---|");
        for (w, _, c, what) in &passed {
            let _ = writeln!(markers, "| {} | {} | {} |", cell(w), code(c), cell(what));
        }
    }
    let _ = writeln!(markers, "\n**`no_check` and `NO_FUCKING_CHECKS`** ({}): each rests on your word.\n", unchecked.len());
    if !unchecked.is_empty() {
        let _ = writeln!(markers, "| Where | Word | Statement | What it covers |\n|---|---|---|---|");
        for (w, word, c, what) in &unchecked {
            let _ = writeln!(markers, "| {} | `{}` | {} | {} |", cell(w), word, code(c), cell(what));
        }
    }
    let _ = writeln!(markers, "\n**Certain failures under your word** ({}):\n", f.analysis.certain_under_nocheck.len());
    for (fi, sid, what) in &f.analysis.certain_under_nocheck {
        let func = &p.functions[*fi];
        let site = func.site(*sid);
        let _ = writeln!(markers, "- {}: `{}` covers something that fails every time it runs: {}", where_(func.span(site.span)), tier_word(site.tier), what);
    }
    sec.push((markers, n_markers));

    // --------------------------------------------------- Types Precog picked
    let mut picked = String::new();
    let mut n_picked = 0;
    let mut prow = Vec::new();
    for (fi, func) in p.functions.iter().enumerate() {
        if !live(fi) || func.nocheat_of.is_some() {
            continue;
        }
        for l in &func.picked {
            let decl = &func.locals[l.0 as usize];
            let span = func.span(decl.span);
            let values = f.analysis.held.get(&(fi, l.0)).map(|d| d.describe()).unwrap_or_else(|| "what its type allows".to_string());
            prow.push(format!("| {} | {} | `'{}'` | `{}` | {} |", cell(&where_(span)), cell(&fname(func)), decl.name.as_deref().unwrap_or("?"), spell(&decl.ty), cell(&values)));
        }
    }
    prow.sort();
    prow.dedup();
    n_picked += prow.len();
    let _ = writeln!(picked, "Each `:=`, the type it got, and every value Precog saw it hold.\n");
    if !prow.is_empty() {
        let _ = writeln!(picked, "| Where | In | Name | Type | Values |\n|---|---|---|---|---|");
        for r in &prow {
            let _ = writeln!(picked, "{}", r);
        }
    }
    sec.push((picked, n_picked));

    // --------------------------------------------------------- Optimizations
    let mut opt = String::new();
    let mut n_opt = 0;
    let readable = |symbol: &str| -> String {
        if symbol == "main" {
            return "the entry, `main`".to_string();
        }
        let n = symbol.strip_prefix("cy_main_").or_else(|| symbol.strip_prefix("cy_").and_then(|r| r.split_once('_').map(|(n, _)| n)));
        match n.and_then(|n| n.parse::<usize>().ok()).and_then(|i| p.functions.get(i)) {
            Some(func) => format!("`{}`", fname(func)),
            None => format!("`{}`", symbol),
        }
    };
    let _ = writeln!(opt, "**Folding**: Precog replaced {} instruction{} by the answer they always give.\n", f.analysis.folded, plural(f.analysis.folded));
    n_opt += f.analysis.folded.min(1);
    let _ = writeln!(opt, "**Inlining**:\n");
    match &f.sizes {
        Ok(s) if !s.inlined.is_empty() => {
            n_opt += s.inlined.len();
            let _ = writeln!(opt, "- Inlined into every caller, taking no room of their own: {}", s.inlined.iter().map(|n| format!("`{}`", n)).collect::<Vec<_>>().join(", "));
        }
        _ => {
            let _ = writeln!(opt, "- No function was inlined into every caller.");
        }
    }
    let _ = writeln!(opt, "- Recursion unrolled: {} call{} of a function to itself (or to a partner calling it back) inlined, up to {} levels and {} instructions a function.\n", f.object.recursion_inlined, plural(f.object.recursion_inlined as usize), crate::codegen::RECURSION_DEPTH, group(crate::codegen::RECURSION_SIZE as u64));
    n_opt += (f.object.recursion_inlined > 0) as usize;
    let tname = |i: usize| -> String {
        let func = &p.functions[i];
        func.inst.and_then(|x| p.instances.get(x.0 as usize)).and_then(|x| p.items.fns.get(x.key.fn_id.0 as usize)).map(|d| d.name.clone()).unwrap_or_else(|| fname(func))
    };
    let _ = writeln!(opt, "**Tables** ({}): pure functions over few enough argument values, every answer worked out while building; each call reads its answer.\n", p.tables.len());
    for t in &p.tables {
        let over = t.ranges.iter().map(|(lo, hi)| format!("{} to {}", lo, hi)).collect::<Vec<_>>().join(" × ");
        let _ = writeln!(opt, "- `{}`: {} entries ({}), {} bytes each, {} in all; the limit is {}, {}", tname(t.function), group(t.entries.len() as u64), over, t.entry_bytes, bytes_text(t.bytes), bytes_text(t.limit), t.limit_from);
    }
    n_opt += p.tables.len();
    let mut nots: Vec<String> = f.analysis.not_tables.iter().map(|n| format!("- `{}`: not a table: {}", tname(n.function), n.why)).collect();
    nots.sort();
    nots.dedup();
    for n in &nots {
        let _ = writeln!(opt, "{}", n);
    }
    let _ = writeln!(opt);
    let _ = writeln!(opt, "**Loops split across cores** ({}):\n", f.plan.loops.len());
    let mut par_rows: Vec<String> = f.plan.loops.iter().map(|((fi, _), pl)| {
        let func = &p.functions[*fi];
        format!("- {} in {}: {} fold{}, about {} unit{} of work a trip; run in slices when trips × work pays for the threads", where_(func.span(pl.fl.span)), code(&fname(func)), pl.folds.len(), plural(pl.folds.len()), pl.work, plural(pl.work as usize))
    }).collect();
    par_rows.sort();
    par_rows.dedup();
    n_opt += par_rows.len();
    for r in &par_rows {
        let _ = writeln!(opt, "{}", r);
    }
    // A loop split in one copy of its function (a `no_cheat` copy, say) is
    // not also said to stay in order.
    let split: HashSet<String> = f.plan.loops.iter().map(|((fi, _), pl)| where_(p.functions[*fi].span(pl.fl.span))).collect();
    let mut kept: Vec<String> = f.plan.declined.iter().filter(|(fi, span, _)| !split.contains(&where_(p.functions[*fi].span(*span)))).map(|(fi, span, why)| format!("- {} in {}: stays in order: {}", where_(p.functions[*fi].span(*span)), code(&fname(&p.functions[*fi])), why)).collect();
    kept.sort();
    kept.dedup();
    if !kept.is_empty() {
        let _ = writeln!(opt, "\n**Loops kept in order** ({}):\n", kept.len());
        for r in &kept {
            let _ = writeln!(opt, "{}", r);
        }
    }
    let mut simd: Vec<String> = f.object.vector_fns.iter().map(|s| readable(s)).collect();
    simd.sort();
    simd.dedup();
    n_opt += simd.len();
    let _ = writeln!(opt, "\n**SIMD**: {}\n", if simd.is_empty() { "no function's optimised code works on vectors.".to_string() } else { format!("the optimised code of {} works on vectors (LLVM's vectorisers, or `simd` values).", simd.join(", ")) });
    let bits = crate::mir::bits::bit_lists(p);
    let mut bit_rows: Vec<String> = bits.iter().map(|(fi, l)| {
        let func = &p.functions[*fi];
        format!("- `'{}'` in {}", func.locals[*l as usize].name.as_deref().unwrap_or("?"), code(&fname(func)))
    }).collect();
    bit_rows.sort();
    bit_rows.dedup();
    n_opt += bit_rows.len();
    let _ = writeln!(opt, "**`list[bool]` held as bits** ({}), eight to a byte: made with `list.fill`, used only by position.\n", bit_rows.len());
    for r in &bit_rows {
        let _ = writeln!(opt, "{}", r);
    }
    let mut regions: Vec<String> = Vec::new();
    for (fi, func) in p.functions.iter().enumerate() {
        if !live(fi) || func.nocheat_of.is_some() {
            continue;
        }
        for (b, _, _) in &func.nocheat_entries {
            let span = func.blocks.get(*b as usize).and_then(|bl| bl.stmts.first().map(|s| s.span).or(Some(bl.term_span))).map(|s| func.span(s));
            if let Some(s) = span {
                regions.push(format!("- {} in {}", where_(s), code(&fname(func))));
            }
        }
        for fl in func.fors.iter().filter(|fl| fl.nocheat) {
            regions.push(format!("- {} in {}: a `no_cheat` loop", where_(func.span(fl.span)), code(&fname(func))));
        }
    }
    regions.sort();
    regions.dedup();
    let copies: Vec<String> = p.functions.iter().enumerate().filter(|(i, g)| g.nocheat_of.is_some() && live(*i)).map(|(_, g)| format!("`{}`", fname(&p.functions[g.nocheat_of.unwrap().0 as usize]))).collect();
    n_opt += regions.len();
    let _ = writeln!(opt, "\n**`no_cheat`** ({} region{}): the work written really runs; nothing folded, dropped or shared there.\n", regions.len(), plural(regions.len()));
    for r in &regions {
        let _ = writeln!(opt, "{}", r);
    }
    if !copies.is_empty() {
        let _ = writeln!(opt, "\nCopies made for it, which run the same way: {}", copies.join(", "));
    }
    sec.push((opt, n_opt));

    // --------------------------------------------------------------- Removed
    let mut removed = String::new();
    let reached = reached(p);
    let mut dropped: Vec<String> = p.functions.iter().filter(|g| g.dropped).map(|g| format!("`{}`", g.name)).collect();
    dropped.sort();
    dropped.dedup();
    let mut never: Vec<String> = p.functions.iter().enumerate().filter(|(i, g)| !g.dropped && !g.is_test && !reached.contains(i) && g.nocheat_of.is_none() && p.main.is_some()).map(|(_, g)| format!("`{}`", fname(g))).collect();
    never.sort();
    never.dedup();
    let tests_out: Vec<String> = if f.tests { Vec::new() } else { p.functions.iter().filter(|g| g.is_test).map(|g| format!("`{}`", g.name)).collect() };
    let _ = writeln!(removed, "Code never reached is not in the program.\n");
    let _ = writeln!(removed, "- Functions of std nothing reaches ({}): {}", dropped.len(), if dropped.is_empty() { "none".to_string() } else { dropped.join(", ") });
    let _ = writeln!(removed, "- This program's functions nothing calls ({}): {}", never.len(), if never.is_empty() { "none".to_string() } else { never.join(", ") });
    let _ = writeln!(removed, "- `test fn`s, left out of a normal build ({}): {}", tests_out.len(), if tests_out.is_empty() { "none".to_string() } else { tests_out.join(", ") });
    sec.push((removed, dropped.len() + never.len() + tests_out.len()));

    // --------------------------------------------------------------------- C
    let mut c = String::new();
    let fns: Vec<_> = p.items.externs.iter().filter(|e| !e.is_let).collect();
    let lets: Vec<_> = p.items.externs.iter().filter(|e| e.is_let).collect();
    // The C functions a live function calls.
    let mut called: HashSet<u32> = HashSet::new();
    for (i, func) in p.functions.iter().enumerate() {
        if live(i) {
            for b in &func.blocks {
                if let Terminator::Call { callee: crate::mir::Callee::Extern(id), .. } = &b.term {
                    called.insert(id.0);
                }
            }
        }
    }
    let used = p.items.externs.iter().enumerate().filter(|(i, e)| !e.is_let && called.contains(&(*i as u32))).count();
    let _ = writeln!(c, "**`extern.C` functions** ({}, {} called), with the CyborgPL types C's map to:\n", fns.len(), used);
    if !fns.is_empty() {
        let _ = writeln!(c, "| Name | Called | Parameters | Returns | Declared at |\n|---|---|---|---|---|");
        let mut rows: Vec<(bool, String)> = Vec::new();
        for (i, e) in p.items.externs.iter().enumerate().filter(|(_, e)| !e.is_let) {
            let params = e.params.iter().map(|(n, t)| format!("'{}': {}", n, spell(t))).chain(e.variadic.then(|| "…".to_string())).collect::<Vec<_>>().join(", ");
            let is = called.contains(&(i as u32));
            rows.push((!is, format!("| `{}` | {} | {} | {} | {} |", e.name, if is { "yes" } else { "no" }, code(&params), code(&format!("{}{}", spell(&e.ret), if e.ranged { " (checked at the door)" } else { "" })), cell(&where_(e.span)))));
        }
        // The ones called first.
        rows.sort_by_key(|r| r.0);
        for (_, r) in rows {
            let _ = writeln!(c, "{}", r);
        }
    }
    let _ = writeln!(c, "\n**C globals** ({}):\n", lets.len());
    for e in &lets {
        let _ = writeln!(c, "- `{}`: `{}`{}", e.name, spell(&e.ret), if e.mutable { ", written" } else { "" });
    }
    let _ = writeln!(c, "\n**C headers read** ({}):\n", f.c_lines.len());
    for (line, n) in f.c_lines {
        let _ = writeln!(c, "- `{}`: {} declaration{}", line, n, plural(*n));
    }
    sec.push((c, fns.len() + lets.len() + f.c_lines.len()));

    // ----------------------------------------------------------------- Sizes
    let mut sizes = String::new();
    let mut n_sizes = 0;
    let debug_bytes = f.debug.map(dir_size);
    match &f.sizes {
        Ok(s) => {
            let _ = writeln!(sizes, "**Totals**:\n\n| What | Bytes |\n|---|---:|");
            let _ = writeln!(sizes, "| The program on disk | {} |", group(s.on_disk));
            let _ = writeln!(sizes, "| Its own code | {} |", group(s.own));
            let _ = writeln!(sizes, "| Its data | {} |", group(s.data));
            if s.bare {
                let _ = writeln!(sizes, "| The runtime's code, {} function{} | {} |", s.runtime_fns, plural(s.runtime_fns), group(s.runtime));
                let _ = writeln!(sizes, "| The file's own layout | {} |", group(s.on_disk.saturating_sub(s.own + s.data + s.runtime)));
            } else {
                let _ = writeln!(sizes, "| The runtime and what the linker adds | {} |", group(s.on_disk.saturating_sub(s.own + s.data)));
            }
            match debug_bytes {
                Some(d) => {
                    let _ = writeln!(sizes, "| Debug info, apart in `{}` | {} |", f.debug.map(|d| d.display().to_string()).unwrap_or_default(), group(d));
                }
                None => {
                    let _ = writeln!(sizes, "| Debug info | inside the program (a working build) |");
                }
            }
            let _ = writeln!(sizes, "\n**Every function** ({} with code of its own, {} inlined):\n", s.rows.len(), s.inlined.len());
            let _ = writeln!(sizes, "| Kit | Function | Bytes | Note |\n|---|---|---:|---|");
            for k in s.kits() {
                for r in s.rows_of(&k) {
                    let _ = writeln!(sizes, "| {} | {} | {} | {} |", cell(&k), code(&r.name), group(r.bytes), cell(s.copy_note(r).trim()));
                }
            }
            for n in &s.inlined {
                let _ = writeln!(sizes, "| | {} | 0 | inlined into every caller |", code(n));
            }
            n_sizes += s.rows.len() + s.inlined.len();
        }
        Err(e) => {
            let _ = writeln!(sizes, "Not measured: {}", e);
        }
    }
    let _ = writeln!(sizes, "\n**Tables** ({}, {} in all):\n", p.tables.len(), bytes_text(p.tables.iter().map(|t| t.bytes).sum()));
    if !p.tables.is_empty() {
        let _ = writeln!(sizes, "| Function | Entries | Bytes |\n|---|---:|---:|");
        for t in &p.tables {
            let _ = writeln!(sizes, "| `{}` | {} | {} |", tname(t.function), group(t.entries.len() as u64), group(t.bytes));
        }
    }
    n_sizes += p.tables.len();
    let _ = writeln!(sizes);
    let _ = writeln!(sizes, "**Consts** ({}):\n", f.consts.len());
    if !f.consts.is_empty() {
        let _ = writeln!(sizes, "| Name | Type | Bytes |\n|---|---|---:|");
        for (n, t, b) in f.consts {
            let _ = writeln!(sizes, "| `'{}'` | {} | {} |", n, cell(t), group(*b));
        }
    }
    n_sizes += f.consts.len();
    let texts = text_literals(p, &live);
    let _ = writeln!(sizes, "\n**Text literals** ({}, {} in all):\n", texts.len(), bytes_text(texts.iter().map(|t| t.len() as u64).sum()));
    if !texts.is_empty() {
        let _ = writeln!(sizes, "| Text | Bytes |\n|---|---:|");
        for t in &texts {
            let _ = writeln!(sizes, "| {} | {} |", code(&shown(t)), group(t.len() as u64));
        }
    }
    n_sizes += texts.len();
    let _ = writeln!(sizes, "\n**Structs** ({}), as `{}` lays them out:\n", f.structs.len(), crate::target::name());
    for s in f.structs {
        let pad: u64 = s.fields.iter().map(|x| x.4).sum();
        let _ = writeln!(sizes, "- `{}`: {}, aligned to {}, {} of padding\n", s.name, bytes_text(s.size), s.align, bytes_text(pad));
        if !s.fields.is_empty() {
            let _ = writeln!(sizes, "  | Field | Type | Offset | Bytes | Padding after |\n  |---|---|---:|---:|---:|");
            for (n, t, at, b, after) in &s.fields {
                let _ = writeln!(sizes, "  | `{}` | {} | {} | {} | {} |", n, code(t), at, b, after);
            }
            let _ = writeln!(sizes);
        }
    }
    n_sizes += f.structs.len();
    sec.push((sizes, n_sizes));

    // ---------------------------------------------------------------- Timing
    let mut timing = String::new();
    let _ = writeln!(timing, "| Phase | Time |\n|---|---:|");
    for (name, d) in f.phases {
        let _ = writeln!(timing, "| {} | {} |", name, secs(*d));
    }
    let _ = writeln!(timing, "| **Total** | **{}** |", secs(f.total));
    sec.push((timing, f.phases.len()));

    // ------------------------------------------------------------------ Notes
    // Every note, the terminal has none (owner, 2026-10-22).
    let mut notes = String::new();
    if f.notes.is_empty() {
        let _ = writeln!(notes, "None.");
    }
    for n in f.notes {
        let _ = writeln!(notes, "```\n{}\n```", n.trim_end());
    }
    sec.push((notes, f.notes.len()));

    // --------------------------------------------------------------- Summary
    let mut out = String::new();
    let _ = writeln!(out, "# Build report: {}\n", f.program.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
    let _ = writeln!(out, "{}\n", SECTIONS.iter().map(|s| format!("[{}](#{})", s, anchor(s))).collect::<Vec<_>>().join(" · "));
    let _ = writeln!(out, "## Summary\n");
    let _ = writeln!(out, "| | |\n|---|---|");
    let _ = writeln!(out, "| Result | built `{}` from `{}` |", f.program.display(), f.source.display());
    let _ = writeln!(out, "| Build | {} |", if f.final_build { "final" } else { "working" });
    let _ = writeln!(out, "| Target | `{}` |", crate::target::name());
    let _ = writeln!(out, "| kayo | `CyDT {} KAYO\\n` |", env!("KAYO_COMMIT"));
    let _ = writeln!(out, "| Date | {} |", utc(f.started));
    let _ = writeln!(out, "| Total time | {} |", secs(f.total));
    let what = ["files read", "sites", "markers", "types picked", "optimizations", "removed", "C names", "sizes listed", "phases", "notes"];
    for (i, (_, n)) in sec.iter().enumerate() {
        let _ = writeln!(out, "| [{}](#{}) | {} {} |", SECTIONS[i + 1], anchor(SECTIONS[i + 1]), n, what[i]);
    }
    let _ = writeln!(out);
    for (i, (body, _)) in sec.iter().enumerate() {
        let _ = writeln!(out, "## {}\n\n{}", SECTIONS[i + 1], body.trim_end());
        let _ = writeln!(out);
    }
    out
}

/// The terminal's one line.
pub fn summary_line(f: &Facts, report: &Path) -> String {
    let sites = f.analysis.sites.len();
    let proven = f.analysis.sites.iter().filter(|r| matches!(r.verdict, Verdict::Proven { .. })).count();
    let trusted = f.analysis.sites.iter().filter(|r| matches!(r.verdict, Verdict::Trusted)).count();
    let checked = sites - proven - trusted;
    let size = match &f.sizes {
        Ok(s) => format!(", {}", bytes_text(s.on_disk)),
        Err(_) => String::new(),
    };
    let marks = if trusted > 0 { format!(", {} no_check", trusted) } else { String::new() };
    format!("built {} ({} build, {}): {} site{}: {} proven, {} checked at run time{}{}, in {}; report {}\n", f.program.display(), if f.final_build { "final" } else { "working" }, crate::target::name(), sites, plural(sites), proven, checked, marks, size, secs(f.total), report.display())
}

/// Every function a body names, from `MAIN` and the tests.
fn reached(p: &Program) -> HashSet<usize> {
    let mut seen = HashSet::new();
    let mut work: Vec<usize> = p.main.map(|m| m.0 as usize).into_iter().chain((0..p.functions.len()).filter(|&f| p.functions[f].is_test)).collect();
    while let Some(f) = work.pop() {
        if f >= p.functions.len() || !seen.insert(f) {
            continue;
        }
        let text = format!("{:?}", p.functions[f].blocks);
        work.extend(text.match_indices("FnIdx(").filter_map(|(at, w)| text[at + w.len()..].split(')').next().and_then(|d| d.parse::<usize>().ok())));
        if let Some(o) = p.functions[f].nocheat_of {
            work.push(o.0 as usize);
        }
    }
    seen
}

/// Every text literal a live function holds, once each.
fn text_literals(p: &Program, live: &dyn Fn(usize) -> bool) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut take = |o: &Operand| {
        if let Operand::Const(Const::Str(s)) = o {
            if !out.contains(s) {
                out.push(s.clone());
            }
        }
    };
    for (i, func) in p.functions.iter().enumerate() {
        if !live(i) {
            continue;
        }
        for b in &func.blocks {
            for s in &b.stmts {
                match &s.kind {
                    StmtKind::Assign { rvalue, .. } => match rvalue {
                        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => take(o),
                        Rvalue::Binary { lhs, rhs, .. } => {
                            take(lhs);
                            take(rhs);
                        }
                        Rvalue::MakeList(os) | Rvalue::MakeStruct { fields: os, .. } => os.iter().for_each(&mut take),
                        Rvalue::MakeMap(kv) => kv.iter().for_each(|(k, v)| {
                            take(k);
                            take(v);
                        }),
                        Rvalue::MakeEnum { payload: Some(o), .. } => take(o),
                        _ => {}
                    },
                    StmtKind::Op { args, .. } => args.iter().for_each(&mut take),
                    _ => {}
                }
            }
            if let Terminator::Call { args, .. } = &b.term {
                for a in args {
                    if let crate::mir::Arg::Value(o) = a {
                        take(o);
                    }
                }
            }
        }
    }
    out.sort();
    out
}

fn tier_word(t: Option<Tier>) -> &'static str {
    match t {
        Some(Tier::Check) => "check",
        Some(Tier::Nocheck) => "no_check",
        Some(Tier::NoChecks) => "NO_FUCKING_CHECKS",
        None => "",
    }
}

fn kind_name(k: &SiteKind, spell: &dyn Fn(&Ty) -> String) -> String {
    match k {
        SiteKind::Overflow(op) => format!("overflow (`{}`)", op.spelled()),
        SiteKind::DivZero => "division by zero".into(),
        SiteKind::ModZero => "remainder by zero".into(),
        SiteKind::Bounds => "position in the list".into(),
        SiteKind::Narrow { to } => format!("conversion into `{}`", spell(to)),
        SiteKind::LengthBound { to } => format!("length bound of `{}`", spell(to)),
        SiteKind::ShiftWidth => "shift past the width".into(),
        SiteKind::ShiftCount => "shift count".into(),
        SiteKind::NegativeExponent => "negative exponent".into(),
        SiteKind::FillCount => "`list.fill` count".into(),
        SiteKind::Position => "text position".into(),
        SiteKind::SliceRange => "slice range".into(),
        SiteKind::SimdRange { lanes } => format!("SIMD lane range ({})", lanes),
        SiteKind::SimdSome { lanes } => format!("SIMD lane present ({})", lanes),
        SiteKind::KeyHashAgrees => "key's hash agrees with equality".into(),
        SiteKind::LendOutlives => "a lend outlives its owner".into(),
        SiteKind::TaskLendOutlives => "a task's lend outlives its owner".into(),
        SiteKind::ChannelReachable => "channel still reachable".into(),
        SiteKind::Deadlock => "deadlock".into(),
    }
}

/// A heading's anchor, as GitHub and most viewers make it.
fn anchor(h: &str) -> String {
    h.to_lowercase().chars().filter_map(|c| if c.is_alphanumeric() { Some(c) } else if c == ' ' || c == '-' { Some('-') } else { None }).collect()
}

/// A path as given, or without its `..` steps where it has some.
fn tidy(p: &Path) -> String {
    if p.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        if let Ok(c) = std::fs::canonicalize(p) {
            return c.display().to_string();
        }
    }
    p.display().to_string()
}

/// Text that is safe inside a table cell.
fn cell(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

/// Code in backticks, safe inside a table cell; `—` for nothing.
fn code(s: &str) -> String {
    if s.is_empty() {
        return "—".into();
    }
    let s = cell(s);
    if s.contains('`') {
        format!("`` {} ``", s)
    } else {
        format!("`{}`", s)
    }
}

/// A text literal as it can be shown on one line.
fn shown(s: &str) -> String {
    let t: String = s.escape_debug().collect();
    if t.chars().count() > 60 {
        format!("\"{}…\"", t.chars().take(60).collect::<String>())
    } else {
        format!("\"{}\"", t)
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// `12,345`.
fn group(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn secs(d: Duration) -> String {
    let s = d.as_secs_f64();
    if s < 0.001 { format!("{:.0} µs", s * 1e6) } else if s < 1.0 { format!("{:.1} ms", s * 1e3) } else { format!("{:.2} s", s) }
}

/// The bytes of a file, or of everything under a folder (`fib.dSYM`).
fn dir_size(p: &Path) -> u64 {
    match std::fs::metadata(p) {
        Ok(m) if m.is_dir() => std::fs::read_dir(p).map(|r| r.filter_map(|e| e.ok()).map(|e| dir_size(&e.path())).sum()).unwrap_or(0),
        Ok(m) => m.len(),
        Err(_) => 0,
    }
}

/// `2026-10-19 08:41:07 UTC`.
fn utc(t: SystemTime) -> String {
    let secs = t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + (m <= 2) as i64;
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", y, m, d, rem / 3600, rem % 3600 / 60, rem % 60)
}
