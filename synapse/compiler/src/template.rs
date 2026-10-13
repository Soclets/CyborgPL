//! A collection's templates (owner, 2026-10-13): `for 'n' in [2, 4] { named
//! "times_" ++ to.stored.str.utf8['n']; ['x': int32] -> int32 { … } }`. What
//! a template makes is written into the file under it by `synapse generate`,
//! below `#generated_N_lines_below`; every build only reads that part, as
//! ordinary members, and refuses it when it no longer matches the template.
//!
//! A template is read as text, as the oracle reads it: its member is the
//! template's own text with the loop's name replaced by each value as
//! written (a const stays its name, `'big'`), outside text, while `named`
//! sees the value itself (`times_100`); a type's name is its text.

use std::collections::HashMap;

use crate::ast::*;
use crate::diag::{err, Diagnostic, Result};

/// A value a template runs over.
#[derive(Clone)]
enum Val {
    Int(i128),
    Text(String),
    Type(String),
}

impl Val {
    fn text(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Text(s) | Val::Type(s) => s.clone(),
        }
    }
}

/// The file's consts whose values are known before building.
fn consts(items: &[Item], out: &mut HashMap<String, Val>) {
    for it in items {
        match &it.kind {
            ItemKind::Const(d) => {
                if let Some(v) = literal(&d.value) {
                    out.insert(d.name.name.clone(), v);
                }
            }
            ItemKind::Module { items, .. } => consts(items, out),
            _ => {}
        }
    }
}

fn literal(e: &Expr) -> Option<Val> {
    match &e.kind {
        ExprKind::Int(t) => t.replace('_', "").parse().ok().map(Val::Int),
        ExprKind::Str(s) => Some(Val::Text(s.clone())),
        ExprKind::Unary { op: UnOp::Neg, expr } => match literal(expr)? {
            Val::Int(n) => Some(Val::Int(-n)),
            _ => None,
        },
        ExprKind::Paren(x) => literal(x),
        _ => None,
    }
}

/// One of a template's values, as known and as written.
fn value(e: &Expr, consts: &HashMap<String, Val>) -> Result<Val> {
    if let Some(v) = literal(e) {
        return Ok(v);
    }
    match &e.kind {
        ExprKind::Name(q) => consts.get(&q.name).cloned().ok_or_else(|| Box::new(Diagnostic::error(format!("'{}' is not known before building: a template runs over literals, consts and type names", q.name)).at(e.span))),
        ExprKind::Path(p) if p.limb.is_none() => Ok(Val::Type(p.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join("."))),
        _ => err(Diagnostic::error("a template runs over values known before building: literals, consts and type names").at(e.span)),
    }
}

/// The name `named` works out, with the loop's name standing for `v`.
fn named(e: &Expr, var: &str, v: &Val, consts: &HashMap<String, Val>) -> Result<String> {
    match &e.kind {
        ExprKind::Str(s) => Ok(s.clone()),
        ExprKind::Paren(x) => named(x, var, v, consts),
        ExprKind::Binary { op: BinOp::Join, lhs, rhs, .. } => Ok(named(lhs, var, v, consts)? + &named(rhs, var, v, consts)?),
        ExprKind::Name(q) if q.name == var => Ok(v.text()),
        ExprKind::Name(q) if consts.contains_key(&q.name) => Ok(consts[&q.name].text()),
        // `to.stored.str.utf8['n']`, `to.str.utf8['n']`: a number's digits.
        ExprKind::Conv { target: Type { kind: TypeKind::Path { path, .. }, .. }, expr, .. } if path.segs.len() >= 2 && path.segs[path.segs.len() - 2].name == "str" && path.last().name == "utf8" => named(expr, var, v, consts),
        _ => err(Diagnostic::error("not yet: Synapse works out a `named` from text, `++`, `to.stored.str.utf8[…]`, the loop's value and consts; a function called there is not built").at(e.span)),
    }
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

fn line_of(text: &str, at: usize) -> usize {
    text[..at].bytes().filter(|b| *b == b'\n').count() + 1
}

fn indent_of(text: &str, at: usize) -> String {
    let s = line_start(text, at);
    text[s..].chars().take_while(|c| *c == ' ' || *c == '\t').collect()
}

/// The values a template runs over, each with how it is written.
fn values(t: &Template, text: &str, consts: &HashMap<String, Val>) -> Result<Vec<(Val, String)>> {
    match &t.values {
        TemplateValues::List(items) => items.iter().map(|e| Ok((value(e, consts)?, text[e.span.start as usize..e.span.end as usize].to_string()))).collect(),
        TemplateValues::Count { start, stop, step, exclusive } => {
            let int = |e: &Expr| match value(e, consts)? {
                Val::Int(n) => Ok(n),
                _ => err(Diagnostic::error("a counting template's bounds are whole numbers").at(e.span)),
            };
            let (a, b, by) = (int(start)?, int(stop)?, int(step)?);
            if by == 0 {
                return err(Diagnostic::error("a step of 0 would never end: the counter must move").at(step.span));
            }
            let mut out = Vec::new();
            let mut n = a;
            while (by > 0 && (n < b || (!exclusive && n == b))) || (by < 0 && (n > b || (!exclusive && n == b))) {
                out.push((Val::Int(n), n.to_string()));
                if out.len() > 10_000 {
                    return err(Diagnostic::error("a template makes at most 10 000 members").at(t.at));
                }
                n += by;
            }
            Ok(out)
        }
    }
}

/// What `synapse generate` writes under a template: the marker line, then
/// each member, indented as the template is.
fn block(t: &Template, text: &str, consts: &HashMap<String, Val>) -> Result<Vec<String>> {
    let base = indent_of(text, t.at.start as usize);
    let member = &text[t.member.start as usize..t.member.end as usize];
    let deeper = indent_of(text, t.member.start as usize).len().saturating_sub(base.len());
    let toks = crate::lexer::lex(crate::source::FileId(0), member)?;
    let mut lines = Vec::new();
    for (v, written) in values(t, text, consts)? {
        let name = named(&t.named, &t.var.name, &v, consts)?;
        let mut body = member.to_string();
        for tok in toks.iter().rev() {
            if matches!(&tok.tok, crate::lexer::Tok::Name(n) if *n == t.var.name) {
                body.replace_range(tok.span.start as usize..tok.span.end as usize, &written);
            }
        }
        for (i, l) in body.lines().enumerate() {
            if i == 0 {
                lines.push(format!("{}{} {}", base, name, l));
            } else {
                let cut = l.chars().take(deeper).take_while(|c| *c == ' ' || *c == '\t').count();
                lines.push(format!("{}{}", base, &l[cut..]));
            }
        }
    }
    lines.insert(0, format!("{}#generated_{}_lines_below", base, lines.len()));
    Ok(lines)
}

/// Every build: each template's generated part is the one its template
/// makes now, right under it.
pub fn check(file: &File, text: &str) -> Result<()> {
    let mut known = HashMap::new();
    consts(&file.items, &mut known);
    for t in &file.templates {
        let want = block(t, text, &known)?;
        let all: Vec<&str> = text.lines().collect();
        let close_line = line_of(text, t.close.start as usize);
        let have: Option<Vec<&str>> = match t.marker {
            Some((n, at)) if line_of(text, at.start as usize) == close_line + 1 => {
                let from = close_line;
                all.get(from..from + 1 + n as usize).map(|s| s.to_vec())
            }
            _ => None,
        };
        let matches = have.as_ref().is_some_and(|h| h.len() == want.len() && h.iter().zip(&want).all(|(a, b)| a.trim_end() == b.trim_end()));
        if !matches {
            let at = t.marker.map_or(t.close, |(_, s)| s);
            return err(Diagnostic::error(format!("the generated part under the template at line {} does not match it: run `synapse generate`", line_of(text, t.at.start as usize))).at(at));
        }
    }
    Ok(())
}

/// `synapse generate`: the text with each template's generated part
/// written, or rewritten, right under it.
pub fn generate(file: &File, text: &str) -> Result<String> {
    let mut known = HashMap::new();
    consts(&file.items, &mut known);
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let mut ts: Vec<&Template> = file.templates.iter().collect();
    ts.sort_by_key(|t| std::cmp::Reverse(t.at.start));
    for t in ts {
        let want = block(t, text, &known)?;
        let close_line = line_of(text, t.close.start as usize);
        let old = match t.marker {
            Some((n, at)) if line_of(text, at.start as usize) == close_line + 1 => 1 + n as usize,
            _ => 0,
        };
        let end = (close_line + old).min(lines.len());
        lines.splice(close_line..end, want);
    }
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}
