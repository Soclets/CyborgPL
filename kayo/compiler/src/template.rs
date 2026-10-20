//! A collection's templates (owner, 2026-10-13): `for 'n' in [2, 4] { named
//! "times_" ++ to.stored.str.utf8['n']; ['x': i32] -> i32 { … } }`. What
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

/// A value a template runs over, or one a function run while generating
/// works with.
#[derive(Clone, PartialEq)]
enum Val {
    Int(i128),
    Text(String),
    Type(String),
    Bool(bool),
}

impl Val {
    fn text(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Text(s) | Val::Type(s) => s.clone(),
            Val::Bool(b) => b.to_string(),
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
        ExprKind::Path(p) if p.kit.is_none() => Ok(Val::Type(p.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join("."))),
        _ => err(Diagnostic::error("a template runs over values known before building: literals, consts and type names").at(e.span)),
    }
}

/// The name `named` works out, with the loop's name standing for `v`: an
/// expression run as a small function returning `str.utf8` (design claude's
/// reading), so `named "times_" ++ to.stored.str.utf8['n'];` and `named
/// size_name['n'];` (owner, 2026-10-13), the function using ordinary `if`s.
fn named(e: &Expr, var: &str, v: &Val, file: &File, consts: &HashMap<String, Val>) -> Result<String> {
    let mut run = Run { file, consts, steps: 0 };
    let mut env = vec![HashMap::from([(var.to_string(), v.clone())])];
    match run.expr(e, &mut env)? {
        Val::Text(s) | Val::Type(s) => Ok(s),
        other => err(Diagnostic::error(format!("`named` works out a name as text, and this gives {}", other.text())).at(e.span)),
    }
}

/// A function of the file run while generating: whole numbers, `bool`,
/// text, `if`, `while`, `loop`, `let`, `change`, `return`, and calls to the
/// file's other functions.
struct Run<'a> {
    file: &'a File,
    consts: &'a HashMap<String, Val>,
    steps: u64,
}

enum Flow {
    Next,
    Return(Val),
    Break,
    Continue,
}

const RUN_STEPS: u64 = 1_000_000;

fn find_fn<'a>(items: &'a [Item], name: &str) -> Option<&'a FnDecl> {
    for it in items {
        match &it.kind {
            ItemKind::Fn(d) if d.owner.is_none() && d.name.name == name => return Some(d),
            ItemKind::ExternBlock(lines) => {
                for (_, k) in lines {
                    if let ItemKind::Fn(d) = k {
                        if d.owner.is_none() && d.name.name == name {
                            return Some(d);
                        }
                    }
                }
            }
            ItemKind::Module { items, .. } => {
                if let Some(d) = find_fn(items, name) {
                    return Some(d);
                }
            }
            _ => {}
        }
    }
    None
}

impl Run<'_> {
    fn lookup(&self, env: &[HashMap<String, Val>], name: &str) -> Option<Val> {
        env.iter().rev().find_map(|s| s.get(name).cloned()).or_else(|| self.consts.get(name).cloned())
    }

    fn not_yet<T>(e: &Expr) -> Result<T> {
        err(Diagnostic::error("not yet: a function run while generating works with whole numbers, `bool` and text, `if`, `while`, `let`, `change` and `return`; this is past it").at(e.span))
    }

    fn int(&mut self, e: &Expr, env: &mut Vec<HashMap<String, Val>>) -> Result<i128> {
        match self.expr(e, env)? {
            Val::Int(n) => Ok(n),
            _ => err(Diagnostic::error("a whole number is wanted here").at(e.span)),
        }
    }

    fn expr(&mut self, e: &Expr, env: &mut Vec<HashMap<String, Val>>) -> Result<Val> {
        self.steps += 1;
        if self.steps > RUN_STEPS {
            return err(Diagnostic::error("a function run while generating took more than a million steps: does it end?").at(e.span));
        }
        Ok(match &e.kind {
            ExprKind::Int(_) | ExprKind::Str(_) => literal(e).unwrap(),
            ExprKind::Bool(b) => Val::Bool(*b),
            ExprKind::Paren(x) => self.expr(x, env)?,
            ExprKind::Name(q) => match self.lookup(env, &q.name) {
                Some(v) => v,
                None => return err(Diagnostic::error(format!("'{}' is not known while generating", q.name)).at(e.span)),
            },
            ExprKind::Unary { op: UnOp::Neg, expr } => Val::Int(self.int(expr, env)?.checked_neg().ok_or_else(|| Box::new(Diagnostic::error("too big").at(e.span)))?),
            ExprKind::Unary { op: UnOp::Not, expr } => match self.expr(expr, env)? {
                Val::Bool(b) => Val::Bool(!b),
                _ => return Self::not_yet(e),
            },
            ExprKind::Binary { op: BinOp::Join, lhs, rhs, .. } => {
                let (l, r) = (self.expr(lhs, env)?, self.expr(rhs, env)?);
                Val::Text(l.text() + &r.text())
            }
            ExprKind::Binary { op: op @ (BinOp::And | BinOp::Or | BinOp::AndThen | BinOp::OrElse), lhs, rhs, .. } => {
                let Val::Bool(l) = self.expr(lhs, env)? else { return Self::not_yet(e) };
                let and = matches!(op, BinOp::And | BinOp::AndThen);
                if l != and {
                    Val::Bool(l)
                } else {
                    let Val::Bool(r) = self.expr(rhs, env)? else { return Self::not_yet(e) };
                    Val::Bool(r)
                }
            }
            ExprKind::Binary { op: op @ (BinOp::Eq | BinOp::Ne), lhs, rhs, .. } => {
                // A type is its name's text here.
                let as_text = |v: Val| match v {
                    Val::Type(t) => Val::Text(t),
                    v => v,
                };
                let (l, r) = (as_text(self.expr(lhs, env)?), as_text(self.expr(rhs, env)?));
                Val::Bool((l == r) == (*op == BinOp::Eq))
            }
            ExprKind::Binary { op, lhs, rhs, .. } => {
                let (l, r) = (self.int(lhs, env)?, self.int(rhs, env)?);
                let over = || Box::new(Diagnostic::error("a whole number past 128 bits while generating").at(e.span));
                match op {
                    BinOp::Add => Val::Int(l.checked_add(r).ok_or_else(over)?),
                    BinOp::Sub => Val::Int(l.checked_sub(r).ok_or_else(over)?),
                    BinOp::Mul => Val::Int(l.checked_mul(r).ok_or_else(over)?),
                    BinOp::Div(round) | BinOp::Mod(round) => {
                        if r == 0 {
                            return err(Diagnostic::error("a division by 0 while generating").at(e.span));
                        }
                        let q = match round {
                            Round::Floor => l.div_euclid(r) - if r < 0 && l.rem_euclid(r) != 0 { 1 } else { 0 },
                            Round::Ceil => -((-l).div_euclid(r) - if r < 0 && (-l).rem_euclid(r) != 0 { 1 } else { 0 }),
                            Round::Trunc => l / r,
                            _ => return Self::not_yet(e),
                        };
                        if matches!(op, BinOp::Div(_)) {
                            Val::Int(q)
                        } else {
                            Val::Int(l - q * r)
                        }
                    }
                    BinOp::Lt => Val::Bool(l < r),
                    BinOp::Le => Val::Bool(l <= r),
                    BinOp::Gt => Val::Bool(l > r),
                    BinOp::Ge => Val::Bool(l >= r),
                    _ => return Self::not_yet(e),
                }
            }
            // `to.stored.str.utf8['n']`, `to.str.utf8['n']`: the digits.
            ExprKind::Conv { target: Type { kind: TypeKind::Path { path, .. }, .. }, expr, .. } if path.segs.len() >= 2 && path.segs[path.segs.len() - 2].name == "str" && path.last().name == "utf8" => Val::Text(self.expr(expr, env)?.text()),
            ExprKind::Call { path, args, otherwise: None } if path.kit.is_none() && path.segs.len() == 1 => {
                let Some(f) = find_fn(&self.file.items, &path.segs[0].name) else {
                    return err(Diagnostic::error(format!("`{}` is not a function of this file, which is what runs while generating", path.segs[0].name)).at(e.span));
                };
                if args.len() != f.params.len() {
                    return err(Diagnostic::error(format!("`{}` takes {} argument(s)", f.name.name, f.params.len())).at(e.span));
                }
                let mut frame = HashMap::new();
                for (p, a) in f.params.iter().zip(args) {
                    let Arg::Positional(x) = a else { return Self::not_yet(e) };
                    let x = match &x.kind {
                        ExprKind::Verb { expr, .. } => expr,
                        _ => x,
                    };
                    // A type reaches a function as its name's text, `i16`.
                    let v = match self.expr(x, env)? {
                        Val::Type(t) => Val::Text(t),
                        v => v,
                    };
                    frame.insert(p.name.name.clone(), v);
                }
                let Some(body) = &f.body else { return Self::not_yet(e) };
                let mut inner = vec![frame];
                match self.block(body, &mut inner)? {
                    Flow::Return(v) => v,
                    _ => return err(Diagnostic::error(format!("`{}` gave nothing back while generating", f.name.name)).at(e.span)),
                }
            }
            _ => return Self::not_yet(e),
        })
    }

    fn block(&mut self, b: &Block, env: &mut Vec<HashMap<String, Val>>) -> Result<Flow> {
        env.push(HashMap::new());
        let mut flow = Flow::Next;
        for s in &b.stmts {
            flow = self.stmt(s, env)?;
            if !matches!(flow, Flow::Next) {
                break;
            }
        }
        env.pop();
        Ok(flow)
    }

    fn stmt(&mut self, s: &Stmt, env: &mut Vec<HashMap<String, Val>>) -> Result<Flow> {
        Ok(match &s.kind {
            StmtKind::Let { name, value, .. } => {
                let v = self.expr(value, env)?;
                env.last_mut().unwrap().insert(name.name.clone(), v);
                Flow::Next
            }
            StmtKind::Assign { target: Expr { kind: ExprKind::Name(q), .. }, value } => {
                let v = self.expr(value, env)?;
                match env.iter_mut().rev().find(|f| f.contains_key(&q.name)) {
                    Some(f) => {
                        f.insert(q.name.clone(), v);
                    }
                    None => return err(Diagnostic::error(format!("'{}' is not known while generating", q.name)).at(s.span)),
                }
                Flow::Next
            }
            StmtKind::Return(Some(v)) => Flow::Return(self.expr(v, env)?),
            StmtKind::If { cond, then, else_ } => {
                let Val::Bool(c) = self.expr(cond, env)? else { return err(Diagnostic::error("an `if` asks a `bool`").at(cond.span)) };
                if c {
                    self.block(then, env)?
                } else {
                    match else_ {
                        Some(ElseArm::Block(b)) => self.block(b, env)?,
                        Some(ElseArm::If(s)) => self.stmt(s, env)?,
                        None => Flow::Next,
                    }
                }
            }
            StmtKind::While { cond, body } => loop {
                let Val::Bool(c) = self.expr(cond, env)? else { return err(Diagnostic::error("a `while` asks a `bool`").at(cond.span)) };
                if !c {
                    break Flow::Next;
                }
                match self.block(body, env)? {
                    Flow::Return(v) => break Flow::Return(v),
                    Flow::Break => break Flow::Next,
                    _ => {}
                }
            },
            StmtKind::Loop(body) => loop {
                match self.block(body, env)? {
                    Flow::Return(v) => break Flow::Return(v),
                    Flow::Break => break Flow::Next,
                    _ => {}
                }
            },
            StmtKind::Break => Flow::Break,
            StmtKind::Continue => Flow::Continue,
            StmtKind::Block(b) => self.block(b, env)?,
            StmtKind::Expr(e) => {
                self.expr(e, env)?;
                Flow::Next
            }
            _ => return err(Diagnostic::error("not yet: a function run while generating works with whole numbers, `bool` and text, `if`, `while`, `let`, `change` and `return`; this is past it").at(s.span)),
        })
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
fn block(t: &Template, text: &str, file: &File, consts: &HashMap<String, Val>) -> Result<Vec<String>> {
    let base = indent_of(text, t.at.start as usize);
    let member = &text[t.member.start as usize..t.member.end as usize];
    let deeper = indent_of(text, t.member.start as usize).len().saturating_sub(base.len());
    let toks = crate::lexer::lex(crate::source::FileId(0), member)?;
    let mut lines = Vec::new();
    for (v, written) in values(t, text, consts)? {
        let name = named(&t.named, &t.var.name, &v, file, consts)?;
        let mut body = member.to_string();
        for tok in toks.iter().rev() {
            if matches!(&tok.tok, crate::lexer::Tok::Name(n) if *n == t.var.name) {
                body.replace_range(tok.span.start as usize..tok.span.end as usize, &written);
            }
        }
        for (i, l) in body.lines().enumerate() {
            if i == 0 {
                // The collection's name in front (owner, 2026-10-19).
                lines.push(format!("{}{}.{} {}", base, t.coll, name, l));
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
        let want = block(t, text, file, &known)?;
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
            return err(Diagnostic::error(format!("the generated part under the template at line {} does not match it: run `kayo generate`", line_of(text, t.at.start as usize))).at(at));
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
        let want = block(t, text, file, &known)?;
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
