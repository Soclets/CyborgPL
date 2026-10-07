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
        let kind = site.sentence(&|t| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string());
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

/// One JSON object per check site, a line each, for the Precog oracle
/// (`precog.md`, attacks b and c: check claude recomputes every claim).
/// An interface between the two sessions, provisional (`cca/README.md`):
/// integers are decimal strings, because JSON readers commonly lose
/// precision past 2^53 and these reach past 2^64.
///
/// `start`/`end` are the site's expression, 1-based line and column; `kind`
/// is the check; `construct` the operator or what the value goes into;
/// `fits` the type (and `range` its integer range) it must stay inside;
/// `verdict` is proven / checked / refused / nocheck; `layer` says which
/// proved it (1, 2, 3, induction), and `value` what layers 1 and 2 saw of
/// the value the check asks about — "solver" when layer 3 settled it.
/// `function` names the instance: a function checked once per shape
/// (generic, variadic, owning argument) has one per shape, so one source
/// site can carry several claims. There is no per-caller analysis: a
/// function's sites are judged over every call that reaches it, joined.
pub fn render_json(a: &Analysis, p: &Program, sources: &SourceMap) -> String {
    use crate::mir::SiteKind;
    use crate::mir::domain::{Domain, ISet};
    let esc = |s: &str| -> String {
        let mut o = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => o.push_str("\\\""),
                '\\' => o.push_str("\\\\"),
                '\n' => o.push_str("\\n"),
                c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                c => o.push(c),
            }
        }
        o.push('"');
        o
    };
    fn iset(s: &ISet) -> String {
        match s {
            ISet::Exact(v) => format!("{{\"values\":[{}]}}", v.iter().map(|x| format!("\"{}\"", x)).collect::<Vec<_>>().join(",")),
            ISet::Range { lo, hi } => format!("{{\"lo\":\"{}\",\"hi\":\"{}\"}}", lo, hi),
            ISet::Empty => "{\"values\":[]}".into(),
        }
    }
    fn dom(d: &Domain) -> String {
        match d {
            Domain::Ints(s) | Domain::Duration(s) => iset(s),
            Domain::Bins { lo: Some(l), hi: Some(h) } => format!("{{\"lo\":\"{:?}\",\"hi\":\"{:?}\"}}", l, h),
            Domain::Bools(t, f) => format!("{{\"bools\":[{}]}}", [(*t, "true"), (*f, "false")].iter().filter(|(b, _)| *b).map(|(_, s)| format!("\"{}\"", s)).collect::<Vec<_>>().join(",")),
            Domain::Seq { len, .. } | Domain::Map { len, .. } => format!("{{\"length\":{}}}", iset(len)),
            Domain::Empty => "{\"values\":[]}".into(),
            _ => "null".into(),
        }
    }
    let mut out = String::new();
    for r in &a.sites {
        let f = &p.functions[r.function];
        let site = f.site(r.site);
        let span = f.span(site.span);
        let file = sources.get(span.file);
        let (sl, sc) = file.line_col(span.start);
        let (el, ec) = file.line_col(span.end);
        let spell = |t: &crate::types::Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
        let (kind, construct): (&str, String) = match &site.kind {
            SiteKind::Overflow(op) if site.result_ty.as_ref().map_or(false, |t| t.is_exact()) => ("past_64_bits", op.spelled()),
            SiteKind::Overflow(op) => ("overflow", op.spelled()),
            SiteKind::DivZero => ("division_by_zero", "/".into()),
            SiteKind::ModZero => ("modulo_by_zero", "mod".into()),
            SiteKind::Bounds => ("position", "[…]".into()),
            SiteKind::Narrow { to } => ("narrowing", format!("into {}", spell(to))),
            SiteKind::LengthBound { to } => ("length_bound", format!("into {}", spell(to))),
            SiteKind::ShiftWidth => ("shift_width", "shl".into()),
            SiteKind::ShiftCount => ("shift_count", "shl/shr".into()),
            SiteKind::NegativeExponent => ("negative_exponent", "pow".into()),
            SiteKind::FillCount => ("fill_count", "list.fill".into()),
            SiteKind::SliceRange => ("slice_range", "slice".into()),
            SiteKind::SimdRange { .. } => ("simd_range", "simd.load/store".into()),
            SiteKind::SimdSome { .. } => ("simd_some", "simd.load.some/store.some".into()),
            SiteKind::KeyHashAgrees => ("key_hash_agrees", "map".into()),
            other => ("other", format!("{:?}", other)),
        };
        let fits_ty = match &site.kind {
            SiteKind::Narrow { to } | SiteKind::LengthBound { to } => Some(to.clone()),
            _ => site.result_ty.clone(),
        };
        let range = fits_ty.as_ref().and_then(|t| super::smt::int_bounds(t)).map(|(lo, hi)| format!("{{\"lo\":\"{}\",\"hi\":\"{}\"}}", lo, hi)).unwrap_or_else(|| "null".into());
        let fits = fits_ty.as_ref().map(|t| esc(&spell(t))).unwrap_or_else(|| "null".into());
        let (verdict, how) = match &r.verdict {
            Verdict::Proven { how } => ("proven", *how),
            Verdict::Trusted => ("!check", ""),
            Verdict::Fails { .. } | Verdict::Unproven => (if site.tier == Some(Tier::Check) { "refused" } else { "checked" }, ""),
        };
        let layer = if !matches!(r.verdict, Verdict::Proven { .. }) {
            "null".to_string()
        } else if how.contains("layer 3") || r.note.contains("layer 3") {
            "\"3\"".into()
        } else if how.contains("induction") || r.note.contains("induction") {
            "\"induction\"".into()
        } else if how.contains("layer 1") || r.note.contains("layer 1") {
            "\"1\"".into()
        } else {
            "\"2\"".into()
        };
        let value = if layer == "\"3\"" { "\"solver\"".to_string() } else { r.value.as_ref().map(dom).unwrap_or_else(|| "null".into()) };
        let operands = r.operands.iter().map(dom).collect::<Vec<_>>().join(",");
        let _ = writeln!(
            out,
            "{{\"file\":{},\"start\":[{},{}],\"end\":[{},{}],\"function\":{},\"kind\":\"{}\",\"construct\":{},\"fits\":{},\"range\":{},\"verdict\":\"{}\",\"layer\":{},\"how\":{},\"value\":{},\"operands\":[{}]}}",
            esc(&file.display_name()), sl, sc, el, ec, esc(&f.name), kind, esc(&construct), fits, range, verdict, layer, esc(if how.is_empty() { &r.note } else { how }), value, operands
        );
    }
    out
}
