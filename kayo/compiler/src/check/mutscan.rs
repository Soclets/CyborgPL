//! `mut` in a function nobody calls (owner, 2026-10-10: "Shouldn't that
//! error?", then "`a`"): a generic one is never checked, its blanks having
//! no type, but whether a variable changes does not depend on them. So its
//! source is read for the promise alone: each `let mut` and `mut`
//! parameter, and every variable something changes — a `change`, a
//! `lend.mut`, an `inline.ASM` output, a capture by `lend.mut` — found by
//! its name in the scopes around it, so a shadow is another variable, as
//! the oracle reads it too (00f61a5e).

use std::collections::HashMap;

use crate::ast::*;
use crate::source::Span;

/// One variable: its name, where, whether `mut`, and whether changed.
pub struct Var {
    pub name: String,
    pub span: Span,
    pub promised: bool,
    pub changed: bool,
}

pub struct Scan {
    pub vars: Vec<Var>,
    scopes: Vec<HashMap<String, usize>>,
}

impl Default for Scan {
    fn default() -> Self {
        Scan { vars: Vec::new(), scopes: vec![HashMap::new()] }
    }
}

impl Scan {
    /// A variable named here, from now on in this scope.
    pub fn declare(&mut self, name: &str, span: Span, promised: bool) {
        self.vars.push(Var { name: name.to_string(), span, promised, changed: false });
        let i = self.vars.len() - 1;
        self.scopes.last_mut().unwrap().insert(name.to_string(), i);
    }

    fn change(&mut self, name: &str) {
        if let Some(i) = self.scopes.iter().rev().find_map(|s| s.get(name).copied()) {
            self.vars[i].changed = true;
        }
    }

    pub fn params(&mut self, ps: &[Param]) {
        for p in ps {
            self.declare(&p.name.name, p.name.span, p.mutability == Some(Mutability::Mut));
        }
    }

    pub fn block(&mut self, b: &Block) {
        self.scopes.push(HashMap::new());
        for s in &b.stmts {
            self.stmt(s);
        }
        self.scopes.pop();
    }

    /// The variable a place starts from: `'x'`, `'x'.f`, `'x'[pos 2]`.
    fn base(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::Name(n) => {
                let name = n.name.clone();
                self.change(&name);
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
                    self.scopes.push(HashMap::new());
                    if let Some(b) = &a.bind {
                        self.declare(&b.name, b.span, false);
                    }
                    self.block(&a.body);
                    self.scopes.pop();
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
                self.change(&c.name.name);
            }
        }
    }

    fn arms(&mut self, arms: &[MatchArm]) {
        for a in arms {
            self.scopes.push(HashMap::new());
            if let Some(b) = &a.bind {
                self.declare(&b.name, b.span, false);
            }
            self.block(&a.body);
            self.scopes.pop();
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { mutability, name, value, .. } => {
                // The value is read before the new name exists.
                self.expr(value);
                self.declare(&name.name, name.span, *mutability == Some(Mutability::Mut));
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
            StmtKind::For { iter, body, name, pos, .. } => {
                match iter {
                    ForIter::Range { start, stop, step, .. } => {
                        self.expr(start);
                        self.expr(stop);
                        if let Some(st) = step {
                            self.expr(st);
                        }
                    }
                    ForIter::Over(e) => self.expr(e),
                }
                self.scopes.push(HashMap::new());
                self.declare(&name.name, name.span, false);
                if let Some(p) = pos {
                    self.declare(&p.name, p.span, false);
                }
                self.block(body);
                self.scopes.pop();
            }
            StmtKind::Return(e) | StmtKind::Fail { payload: e, .. } => {
                if let Some(e) = e {
                    self.expr(e);
                }
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Tier { stmt, .. } | StmtKind::NoCheat { stmt } => self.stmt(stmt),
            StmtKind::Concur { captures, body, otherwise, .. } => {
                self.captures(captures);
                // Inside, the captures are the task's own names.
                self.scopes.push(HashMap::new());
                for c in captures {
                    self.declare(&c.name.name, c.name.span, false);
                }
                match body {
                    ConcurBody::Block(b) => self.block(b),
                    ConcurBody::Call(e) => self.expr(e),
                }
                self.scopes.pop();
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            StmtKind::Whichever { arms, otherwise } => {
                for a in arms {
                    self.expr(&a.channel);
                    self.scopes.push(HashMap::new());
                    self.declare(&a.name.name, a.name.span, false);
                    self.block(&a.body);
                    self.scopes.pop();
                }
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            StmtKind::Match { value, arms } => {
                self.expr(value);
                self.arms(arms);
            }
            StmtKind::InlineAsm { bindings, .. } => {
                for b in bindings {
                    match b {
                        AsmBinding::Reg { name, dir: AsmDir::InOut, .. } | AsmBinding::Lend { name, mutable: true, .. } => {
                            let name = name.name.clone();
                            self.change(&name);
                        }
                        AsmBinding::Reg { name, dir: AsmDir::Out, .. } => {
                            let (n, at) = (name.name.clone(), name.span);
                            self.declare(&n, at, false);
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
                self.arms(arms);
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
            ExprKind::Unary { expr, .. } | ExprKind::Paren(expr) | ExprKind::Pos(expr) | ExprKind::Count(expr) | ExprKind::Value(expr) | ExprKind::Label(_, expr) => self.expr(expr),
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
                // Its own names: the captures and the parameters.
                self.scopes.push(HashMap::new());
                for c in &lit.captures {
                    self.declare(&c.name.name, c.name.span, false);
                }
                self.params(&lit.params);
                self.block(&lit.body);
                self.scopes.pop();
            }
            _ => {}
        }
    }
}
