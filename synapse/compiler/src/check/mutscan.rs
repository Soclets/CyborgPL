//! `mut` in a function nobody calls (owner, 2026-10-10: "Shouldn't that
//! error?", then "`a`"): a generic one is never checked, its blanks having
//! no type, but whether a variable changes does not depend on them. So its
//! source is read for the promise alone: each `let mut` and `mut`
//! parameter, and every name something changes — a `change`, a
//! `lend.mut`, an `inline.ASM` output, a capture by `lend.mut` — matched by
//! name within the function, as the oracle does.

use std::collections::HashSet;

use crate::ast::*;
use crate::source::Span;

#[derive(Default)]
pub struct Scan {
    pub promised: Vec<(String, Span)>,
    pub changed: HashSet<String>,
}

impl Scan {
    pub fn params(&mut self, ps: &[Param]) {
        for p in ps {
            if p.mutability == Some(Mutability::Mut) {
                self.promised.push((p.name.name.clone(), p.name.span));
            }
        }
    }

    pub fn block(&mut self, b: &Block) {
        for s in &b.stmts {
            self.stmt(s);
        }
    }

    /// The variable a place starts from: `'x'`, `'x'.f`, `'x'[pos 2]`.
    fn base(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Name(n) => {
                self.changed.insert(n.name.clone());
            }
            ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => self.base(base),
            ExprKind::Paren(x) | ExprKind::Pos(x) => self.base(x),
            _ => {}
        }
    }

    fn otherwise(&mut self, o: &Otherwise) {
        match o {
            Otherwise::Value(e) => self.expr(e),
            Otherwise::Leave(s) => self.stmt(s),
            Otherwise::PassUp(_) => {}
            Otherwise::Arms(arms, _) => {
                for a in arms {
                    self.block(&a.body);
                }
            }
        }
    }

    fn args(&mut self, args: &[Arg]) {
        for a in args {
            match a {
                Arg::Positional(e) | Arg::Named(_, e) => self.expr(e),
                Arg::Pair(k, v) => {
                    self.expr(k);
                    self.expr(v);
                }
            }
        }
    }

    fn captures(&mut self, cs: &[Capture]) {
        for c in cs {
            if c.verb == Verb::LendMut {
                self.changed.insert(c.name.name.clone());
            }
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { mutability, name, value, .. } => {
                if *mutability == Some(Mutability::Mut) {
                    self.promised.push((name.name.clone(), name.span));
                }
                self.expr(value);
            }
            StmtKind::Assign { target, value } => {
                self.base(target);
                self.expr(target);
                self.expr(value);
            }
            StmtKind::Const(d) => self.expr(&d.value),
            StmtKind::Expr(e) | StmtKind::Gives { value: e, .. } | StmtKind::Must { expr: e, .. } => self.expr(e),
            StmtKind::If { cond, then, else_ } => {
                self.expr(cond);
                self.block(then);
                match else_ {
                    Some(ElseArm::If(s)) => self.stmt(s),
                    Some(ElseArm::Block(b)) => self.block(b),
                    None => {}
                }
            }
            StmtKind::Loop(b) | StmtKind::Block(b) => self.block(b),
            StmtKind::While { cond, body } => {
                self.expr(cond);
                self.block(body);
            }
            StmtKind::For { iter, body, .. } => {
                match iter {
                    ForIter::Range { start, stop, step } => {
                        self.expr(start);
                        self.expr(stop);
                        if let Some(st) = step {
                            self.expr(st);
                        }
                    }
                    ForIter::Over(e) => self.expr(e),
                }
                self.block(body);
            }
            StmtKind::Return(e) | StmtKind::Fail { payload: e, .. } => {
                if let Some(e) = e {
                    self.expr(e);
                }
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Tier { stmt, .. } => self.stmt(stmt),
            StmtKind::Concur { captures, body, otherwise, .. } => {
                self.captures(captures);
                match body {
                    ConcurBody::Block(b) => self.block(b),
                    ConcurBody::Call(e) => self.expr(e),
                }
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            StmtKind::Whichever { arms, otherwise } => {
                for a in arms {
                    self.expr(&a.channel);
                    self.block(&a.body);
                }
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            StmtKind::Match { value, arms } => {
                self.expr(value);
                for a in arms {
                    self.block(&a.body);
                }
            }
            StmtKind::InlineAsm { bindings, .. } => {
                for b in bindings {
                    match b {
                        AsmBinding::Reg { name, dir: AsmDir::InOut, .. } | AsmBinding::Lend { name, mutable: true, .. } => {
                            self.changed.insert(name.name.clone());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Verb { verb, expr, .. } => {
                if *verb == Verb::LendMut {
                    self.base(expr);
                }
                self.expr(expr);
            }
            ExprKind::Spread { verb, expr } => {
                if *verb == Some(Verb::LendMut) {
                    self.base(expr);
                }
                self.expr(expr);
            }
            ExprKind::Match { value, arms } => {
                self.expr(value);
                for a in arms {
                    self.block(&a.body);
                }
            }
            ExprKind::Call { args, otherwise, .. } => {
                self.args(args);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            ExprKind::CallValue { callee, args, otherwise } => {
                self.expr(callee);
                self.args(args);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            ExprKind::Conv { expr, otherwise, .. } => {
                self.expr(expr);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            ExprKind::Field { base, .. } => self.expr(base),
            ExprKind::Index { base, index } => {
                self.expr(base);
                self.expr(index);
            }
            ExprKind::Unary { expr, .. } | ExprKind::Paren(expr) | ExprKind::Pos(expr) | ExprKind::Count(expr) | ExprKind::Value(expr) => self.expr(expr),
            ExprKind::PtrCast { value, .. } => self.expr(value),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            ExprKind::List(items) => {
                for x in items {
                    self.expr(x);
                }
            }
            ExprKind::Range { start, stop, step } => {
                self.expr(start);
                self.expr(stop);
                if let Some(st) = step {
                    self.expr(st);
                }
            }
            ExprKind::Channel { bound: Some(b), .. } => {
                self.expr(&b.0);
                self.expr(&b.1);
            }
            ExprKind::FnLit(lit) => {
                self.captures(&lit.captures);
                self.params(&lit.params);
                self.block(&lit.body);
            }
            _ => {}
        }
    }
}
