//! Smaller programs to try when a finding is being shrunk. Each candidate is
//! one step smaller; the harness keeps a step only if the finding survives,
//! so a candidate that breaks the program is simply not kept.

use crate::ast::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Only remove or unwrap statements and functions; never touch an
/// expression, a type or a tier. For a `check` CCA could not prove: the
/// generator's clamps are what make it provable, and simplifying an
/// expression can remove one and make the refusal right (seed 58, twice).
pub static STRUCTURAL_ONLY: AtomicBool = AtomicBool::new(false);

fn structural() -> bool {
    STRUCTURAL_ONLY.load(Ordering::Relaxed)
}

pub fn candidates(p: &Program) -> Vec<Program> {
    let mut out = vec![];
    for i in 0..p.funcs.len() {
        let mut q = p.clone();
        q.funcs.remove(i);
        out.push(q);
    }
    for v in block_variants(&p.main) {
        out.push(Program { main: v, ..p.clone() });
    }
    for (i, f) in p.funcs.iter().enumerate() {
        // A recursive function starts with its base case and its self-call;
        // changing either can make it endless.
        let keep = if calls_itself(f) { 2 } else { 0 };
        for v in block_variants_from(&f.body, keep) {
            let mut q = p.clone();
            q.funcs[i].body = v;
            out.push(q);
        }
    }
    if p.loose && !structural() {
        out.push(Program { loose: false, ..p.clone() });
    }
    // Statement removals first: they shrink fastest.
    out.sort_by_key(|q| size(q));
    out
}

pub fn size(p: &Program) -> usize {
    p.funcs.iter().map(|f| block_size(&f.body) + 1).sum::<usize>() + block_size(&p.main)
}

fn block_size(b: &[Stmt]) -> usize {
    b.iter().map(stmt_size).sum()
}

fn stmt_size(s: &Stmt) -> usize {
    1 + match s {
        Stmt::Let { value, .. } | Stmt::Assign { value, .. } | Stmt::Print { value, .. } | Stmt::Return(value) => expr_size(value),
        Stmt::If { arms, otherwise } => {
            arms.iter().map(|(c, b)| expr_size(c) + block_size(b)).sum::<usize>() + otherwise.as_ref().map_or(0, |b| block_size(b))
        }
        Stmt::For { start, stop, body, .. } => expr_size(start) + expr_size(stop) + block_size(body),
        Stmt::While { cond, body } => expr_size(cond) + block_size(body),
        Stmt::Loop { body } | Stmt::Block { body, .. } => block_size(body),
        Stmt::Break | Stmt::Continue => 0,
    }
}

fn expr_size(e: &Expr) -> usize {
    1 + match e {
        Expr::Lit(_) | Expr::Bool(_) | Expr::Var(_) => 0,
        Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) => expr_size(a) + expr_size(b),
        Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) => expr_size(a),
        Expr::Call(_, args) => args.iter().map(expr_size).sum(),
    }
}

fn block_variants(b: &[Stmt]) -> Vec<Vec<Stmt>> {
    block_variants_from(b, 0)
}

/// Variants that leave the first `keep` statements alone: a generated
/// `while` starts with its counter's decrement and a `loop` with its break
/// test and decrement, and changing those can make the loop endless.
fn block_variants_from(b: &[Stmt], keep: usize) -> Vec<Vec<Stmt>> {
    let mut out = vec![];
    for i in keep.min(b.len())..b.len() {
        let mut v = b.to_vec();
        v.remove(i);
        out.push(v);
    }
    for i in keep.min(b.len())..b.len() {
        for rep in stmt_variants(&b[i]) {
            let mut v = b[..i].to_vec();
            v.extend(rep);
            v.extend_from_slice(&b[i + 1..]);
            out.push(v);
        }
    }
    out
}

/// Replacements for one statement (each a list, so a statement can be
/// replaced by the statements inside it).
fn stmt_variants(s: &Stmt) -> Vec<Vec<Stmt>> {
    let mut out: Vec<Vec<Stmt>> = vec![];
    match s {
        Stmt::Let { .. } | Stmt::Assign { .. } if structural() => {}
        Stmt::Let { tier, name, mutability, ty, value } => {
            if *tier != Tier::Default {
                out.push(vec![Stmt::Let { tier: Tier::Default, name: name.clone(), mutability: *mutability, ty: *ty, value: value.clone() }]);
            }
            if ty.range.is_some() {
                out.push(vec![Stmt::Let { tier: *tier, name: name.clone(), mutability: *mutability, ty: ty.base.into(), value: value.clone() }]);
            }
            if *mutability != Mutability::Plain {
                out.push(vec![Stmt::Let { tier: *tier, name: name.clone(), mutability: Mutability::Plain, ty: *ty, value: value.clone() }]);
            }
            for e in expr_variants(value) {
                out.push(vec![Stmt::Let { tier: *tier, name: name.clone(), mutability: *mutability, ty: *ty, value: e }]);
            }
        }
        Stmt::Assign { tier, name, value } => {
            if *tier != Tier::Default {
                out.push(vec![Stmt::Assign { tier: Tier::Default, name: name.clone(), value: value.clone() }]);
            }
            for e in expr_variants(value) {
                out.push(vec![Stmt::Assign { tier: *tier, name: name.clone(), value: e }]);
            }
        }
        Stmt::Print { signed, value } => {
            for e in expr_variants(value) {
                out.push(vec![Stmt::Print { signed: *signed, value: e }]);
            }
        }
        Stmt::Return(value) => {
            for e in expr_variants(value) {
                out.push(vec![Stmt::Return(e)]);
            }
        }
        Stmt::If { arms, otherwise } => {
            for (_, body) in arms {
                out.push(body.clone());
            }
            if let Some(b) = otherwise {
                out.push(b.clone());
                out.push(vec![Stmt::If { arms: arms.clone(), otherwise: None }]);
            }
            if arms.len() > 1 {
                for i in 0..arms.len() {
                    let mut a = arms.clone();
                    a.remove(i);
                    out.push(vec![Stmt::If { arms: a, otherwise: otherwise.clone() }]);
                }
            }
            for i in 0..arms.len() {
                for c in expr_variants(&arms[i].0) {
                    let mut a = arms.clone();
                    a[i].0 = c;
                    out.push(vec![Stmt::If { arms: a, otherwise: otherwise.clone() }]);
                }
                for b in block_variants(&arms[i].1) {
                    let mut a = arms.clone();
                    a[i].1 = b;
                    out.push(vec![Stmt::If { arms: a, otherwise: otherwise.clone() }]);
                }
            }
            if let Some(o) = otherwise {
                for b in block_variants(o) {
                    out.push(vec![Stmt::If { arms: arms.clone(), otherwise: Some(b) }]);
                }
            }
        }
        Stmt::For { perm, name, ty, start, stop, step, body } => {
            let mk = |perm: bool, start: Expr, stop: Expr, step: Option<i128>, body: Vec<Stmt>| Stmt::For { perm, name: name.clone(), ty: *ty, start, stop, step, body };
            if *perm {
                out.push(vec![mk(false, start.clone(), stop.clone(), *step, body.clone())]);
            }
            out.push(vec![mk(*perm, start.clone(), start.clone(), *step, body.clone())]);
            if step.is_some() {
                out.push(vec![mk(*perm, start.clone(), stop.clone(), None, body.clone())]);
            }
            for b in block_variants(body) {
                out.push(vec![mk(*perm, start.clone(), stop.clone(), *step, b)]);
            }
        }
        Stmt::While { cond, body } => {
            for b in block_variants_from(body, 1) {
                out.push(vec![Stmt::While { cond: cond.clone(), body: b }]);
            }
        }
        Stmt::Loop { body } => {
            for b in block_variants_from(body, 2) {
                out.push(vec![Stmt::Loop { body: b }]);
            }
        }
        Stmt::Block { tier, body } => {
            out.push(body.clone());
            if *tier != Tier::Default && !structural() {
                out.push(vec![Stmt::Block { tier: Tier::Default, body: body.clone() }]);
            }
            for b in block_variants(body) {
                out.push(vec![Stmt::Block { tier: *tier, body: b }]);
            }
        }
        Stmt::Break | Stmt::Continue => {}
    }
    out
}

/// Simpler expressions to put in `e`'s place: its own parts, small
/// literals, and every simplification of a part.
fn expr_variants(e: &Expr) -> Vec<Expr> {
    let mut out = vec![];
    if structural() {
        return out;
    }
    match e {
        Expr::Lit(v) => {
            if *v != 0 {
                out.push(Expr::Lit(0));
            }
            if *v != 1 && *v != 0 {
                out.push(Expr::Lit(1));
            }
            if v.abs() > 10 {
                out.push(Expr::Lit(v / 2));
            }
            return out;
        }
        Expr::Bool(_) | Expr::Var(_) => return out,
        _ => {}
    }
    let parts: Vec<&Expr> = match e {
        Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) => vec![a, b],
        Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) => vec![a],
        Expr::Call(_, args) => args.iter().collect(),
        _ => vec![],
    };
    for p in &parts {
        out.push((*p).clone());
    }
    match e {
        Expr::Cmp(..) | Expr::And(..) | Expr::Or(..) | Expr::Not(..) => {
            out.push(Expr::Bool(true));
            out.push(Expr::Bool(false));
        }
        _ => {
            out.push(Expr::Lit(0));
            out.push(Expr::Lit(1));
        }
    }
    let rebuild = |i: usize, x: Expr| -> Expr {
        let b = |y: &Expr| Box::new(y.clone());
        match e {
            Expr::Bin(op, a, c) => if i == 0 { Expr::Bin(*op, Box::new(x), b(c)) } else { Expr::Bin(*op, b(a), Box::new(x)) },
            Expr::Cmp(op, a, c) => if i == 0 { Expr::Cmp(*op, Box::new(x), b(c)) } else { Expr::Cmp(*op, b(a), Box::new(x)) },
            Expr::And(a, c) => if i == 0 { Expr::And(Box::new(x), b(c)) } else { Expr::And(b(a), Box::new(x)) },
            Expr::Or(a, c) => if i == 0 { Expr::Or(Box::new(x), b(c)) } else { Expr::Or(b(a), Box::new(x)) },
            Expr::Neg(_) => Expr::Neg(Box::new(x)),
            Expr::NotBit(_) => Expr::NotBit(Box::new(x)),
            Expr::Not(_) => Expr::Not(Box::new(x)),
            Expr::To(t, _) => Expr::To(*t, Box::new(x)),
            Expr::Call(f, args) => {
                let mut a = args.clone();
                a[i] = x;
                Expr::Call(f.clone(), a)
            }
            _ => unreachable!(),
        }
    };
    for (i, p) in parts.iter().enumerate() {
        for v in expr_variants(p) {
            out.push(rebuild(i, v));
        }
    }
    out
}

/// Whether every name, function and call in `p` refers to something declared
/// before it, with the right number of arguments. A cheap filter: most
/// shrinking steps that break a program break it this way.
pub fn well_scoped(p: &Program) -> bool {
    let mut arity: Vec<(String, usize)> = vec![("put_i".into(), 1), ("put_u".into(), 1)];
    for f in &p.funcs {
        let mut scopes = vec![f.params.iter().map(|x| x.name.clone()).collect::<Vec<_>>()];
        arity.push((f.name.clone(), f.params.len()));
        if !block_ok(&f.body, &mut scopes, &arity) {
            return false;
        }
    }
    let mut scopes = vec![p.inputs.iter().map(|i| i.name.clone()).collect::<Vec<_>>()];
    block_ok(&p.main, &mut scopes, &arity)
}

fn known(scopes: &[Vec<String>], n: &str) -> bool {
    scopes.iter().any(|s| s.iter().any(|x| x == n))
}

fn block_ok(b: &[Stmt], scopes: &mut Vec<Vec<String>>, arity: &[(String, usize)]) -> bool {
    scopes.push(vec![]);
    let ok = b.iter().all(|s| stmt_ok(s, scopes, arity));
    scopes.pop();
    ok
}

fn stmt_ok(s: &Stmt, scopes: &mut Vec<Vec<String>>, arity: &[(String, usize)]) -> bool {
    let e = |x: &Expr, scopes: &Vec<Vec<String>>| expr_ok(x, scopes, arity);
    match s {
        Stmt::Let { name, value, .. } => {
            let ok = e(value, scopes);
            scopes.last_mut().unwrap().push(name.clone());
            ok
        }
        Stmt::Assign { name, value, .. } => known(scopes, name) && e(value, scopes),
        Stmt::Print { value, .. } | Stmt::Return(value) => e(value, scopes),
        Stmt::If { arms, otherwise } => {
            arms.iter().all(|(c, b)| e(c, scopes) && block_ok(b, scopes, arity)) && otherwise.as_ref().map_or(true, |b| block_ok(b, scopes, arity))
        }
        Stmt::For { perm, name, start, stop, body, .. } => {
            let ok = e(start, scopes) && e(stop, scopes) && {
                scopes.push(vec![name.clone()]);
                let ok = block_ok(body, scopes, arity);
                scopes.pop();
                ok
            };
            if *perm {
                scopes.last_mut().unwrap().push(name.clone());
            }
            ok
        }
        Stmt::While { cond, body } => e(cond, scopes) && block_ok(body, scopes, arity),
        Stmt::Loop { body } | Stmt::Block { body, .. } => block_ok(body, scopes, arity),
        Stmt::Break | Stmt::Continue => true,
    }
}

fn expr_ok(e: &Expr, scopes: &Vec<Vec<String>>, arity: &[(String, usize)]) -> bool {
    match e {
        Expr::Lit(_) | Expr::Bool(_) => true,
        Expr::Var(n) => known(scopes, n),
        Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) => expr_ok(a, scopes, arity) && expr_ok(b, scopes, arity),
        Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) => expr_ok(a, scopes, arity),
        Expr::Call(f, args) => arity.iter().any(|(n, k)| n == f && *k == args.len()) && args.iter().all(|a| expr_ok(a, scopes, arity)),
    }
}

fn calls_itself(f: &Func) -> bool {
    fn in_expr(e: &Expr, n: &str) -> bool {
        match e {
            Expr::Call(g, args) => g == n || args.iter().any(|a| in_expr(a, n)),
            Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) => in_expr(a, n) || in_expr(b, n),
            Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) => in_expr(a, n),
            _ => false,
        }
    }
    f.body.iter().any(|s| matches!(s, Stmt::Let { value, .. } if in_expr(value, &f.name)))
}
