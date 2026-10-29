//! A `fn` has no effects and calls only `fn`s; a `proc` may do anything (owner, 2026-10-27,
//! redesign two: "a word means what it means in mathematics"). A `fn` may change its own
//! variables, which no caller sees, and one that can fail is still a `fn`. What makes a `proc`:
//! printing, reading input, waiting, exiting, a task or a channel, C or a pointer, another
//! language's code, and calling a `proc` or a `proc[…]` value. Changing what it was lent is no
//! effect (owner, 2026-10-29: "(a)"): the caller handed it over to be changed.
//!
//! Read from a function's checked body, the first such thing found.

use crate::hir::{Arg, Block, Builtin, Callee, Expr, ExprKind, Otherwise, Place, Stmt, StmtKind};
use crate::source::Span;
use crate::types::Ty;

/// The first thing that makes a body a `proc`'s.
pub enum Found {
    /// An effect: what it does, `prints (`std::print`)`.
    Effect(String, Span),
    /// A call of a `proc` by name, `say`.
    Proc(String, Span),
    /// A call of a held `proc[…]` value, `$act`.
    ProcValue(String, Span),
}

pub struct Walk<'a> {
    pub checker: &'a super::Checker,
    pub locals: &'a [crate::hir::Local],
    pub found: Option<Found>,
}

impl<'a> Walk<'a> {
    fn note(&mut self, f: Found) {
        if self.found.is_none() {
            self.found = Some(f);
        }
    }

    pub fn block(&mut self, b: &Block) {
        for s in &b.stmts {
            if self.found.is_some() {
                return;
            }
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match &s.kind {
            StmtKind::NoCheat(ss) => ss.iter().for_each(|s| self.stmt(s)),
            StmtKind::Let { value, .. } => self.expr(value),
            StmtKind::Assign { place, value } => {
                self.place(place);
                self.expr(value);
            }
            StmtKind::Expr(e) => self.expr(e),
            StmtKind::If { cond, then, else_ } => {
                self.expr(cond);
                self.block(then);
                if let Some(b) = else_ {
                    self.block(b);
                }
            }
            StmtKind::Loop(b) | StmtKind::Block(b) => self.block(b),
            StmtKind::While { cond, body } => {
                self.expr(cond);
                self.block(body);
            }
            StmtKind::ForRange { start, stop, step, body, .. } => {
                self.expr(start);
                self.expr(stop);
                if let Some(e) = step {
                    self.expr(e);
                }
                self.block(body);
            }
            StmtKind::ForEach { iter, body, .. } => {
                self.expr(iter);
                self.block(body);
            }
            StmtKind::ForNext { call, body, .. } => {
                self.expr(call);
                self.block(body);
            }
            StmtKind::Return(e) => {
                if let Some(e) = e {
                    self.expr(e);
                }
            }
            StmtKind::Fail { payload, .. } => {
                if let Some(e) = payload {
                    self.expr(e);
                }
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Match { value, arms } => {
                self.expr(value);
                for a in arms {
                    self.block(&a.body);
                    if let Some(g) = &a.gives {
                        self.expr(g);
                    }
                }
            }
            StmtKind::Concur { .. } => self.note(Found::Effect("starts a task (`%spawn.task`)".into(), s.span)),
            StmtKind::Whichever { .. } => self.note(Found::Effect("waits on channels (`%whichever`)".into(), s.span)),
            StmtKind::InlineAsm { msl, .. } => {
                let what = if msl.is_some() { "runs Metal's code (`inline.%MSL`)" } else { "runs assembly (`inline.ASM`)" };
                self.note(Found::Effect(what.into(), s.span));
            }
        }
    }

    fn otherwise(&mut self, o: &Otherwise) {
        match o {
            Otherwise::Value(e) => self.expr(e),
            Otherwise::Leave(s) => self.stmt(s),
            Otherwise::PassUp => {}
            Otherwise::Arms(arms) => {
                for a in arms {
                    self.block(&a.body);
                    if let Some(g) = &a.gives {
                        self.expr(g);
                    }
                }
            }
        }
    }

    fn place(&mut self, p: &Place) {
        for pr in &p.proj {
            if let crate::hir::Proj::Index(e) = pr {
                self.expr(e);
            }
        }
    }

    fn arg(&mut self, a: &Arg, builtin: bool) {
        match a {
            Arg::Value(e) => self.expr(e),
            Arg::Lend { place, .. } => self.place(place),
            Arg::Give(p) | Arg::Copy(p) => self.place(p),
            Arg::Spread { expr, .. } => self.arg(expr, builtin),
        }
    }

    fn expr(&mut self, e: &Expr) {
        if self.found.is_some() {
            return;
        }
        match &e.kind {
            ExprKind::Match { value, arms } => {
                self.expr(value);
                for a in arms {
                    self.block(&a.body);
                    if let Some(g) = &a.gives {
                        self.expr(g);
                    }
                }
            }
            ExprKind::Read(p) | ExprKind::Copy(p) | ExprKind::Give(p) | ExprKind::Lend { place: p, .. } | ExprKind::BundleElem { place: p, .. } => self.place(p),
            ExprKind::ExternRead(_) => self.note(Found::Effect("reads C's variable".into(), e.span)),
            ExprKind::Call { callee, args, otherwise, .. } => {
                let builtin = matches!(callee, Callee::Builtin(_));
                match callee {
                    Callee::Fn(inst) => {
                        let key = &self.checker.instances[inst.0 as usize].key;
                        let decl = &self.checker.items.fns[key.fn_id.0 as usize];
                        if decl.proc {
                            self.note(Found::Proc(decl.name.clone(), e.span));
                        }
                    }
                    Callee::Extern(id) => {
                        let name = self.checker.items.externs.get(id.0 as usize).map(|x| x.name.clone()).unwrap_or_default();
                        self.note(Found::Effect(format!("calls C (`{}`)", name), e.span));
                    }
                    Callee::Value(v) => {
                        if let Ty::Fn(f) = self.checker.resolve(&v.ty).base() {
                            if f.proc {
                                let held = match &v.kind {
                                    ExprKind::Local(l) | ExprKind::Read(Place { base: l, .. }) => self.locals.get(l.0 as usize).map(|x| format!("${}", x.name)).unwrap_or_else(|| "the value".into()),
                                    _ => "the value".into(),
                                };
                                self.note(Found::ProcValue(held, e.span));
                            }
                        }
                        self.expr(v);
                    }
                    Callee::Builtin(b) => {
                        if let Some(what) = builtin_effect(b) {
                            self.note(Found::Effect(what, e.span));
                        }
                    }
                }
                for a in args {
                    self.arg(a, builtin);
                }
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            ExprKind::Construct { fields, .. } | ExprKind::List(fields) => fields.iter().for_each(|a| self.arg(a, false)),
            ExprKind::EnumCase { payload, .. } => {
                if let Some(a) = payload {
                    self.arg(a, false);
                }
            }
            ExprKind::MapLit(pairs) => {
                for (k, v) in pairs {
                    self.arg(k, false);
                    self.arg(v, false);
                }
            }
            ExprKind::Unary { expr, .. } | ExprKind::PtrCast(expr) => self.expr(expr),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs);
                self.expr(rhs);
            }
            ExprKind::UserOp { lhs, rhs, .. } => {
                self.arg(lhs, false);
                self.arg(rhs, false);
            }
            ExprKind::Conv { expr, otherwise, .. } => {
                self.expr(expr);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            ExprKind::Channel { .. } => self.note(Found::Effect("makes a channel (`%channel`)".into(), e.span)),
            _ => {}
        }
    }
}

/// What a language operation does that a `fn` may not.
fn builtin_effect(b: &Builtin) -> Option<String> {
    Some(match b {
        Builtin::PtrRead | Builtin::PtrWrite | Builtin::CStrRead => "reaches through a pointer".into(),
        Builtin::ChannelSend | Builtin::ChannelTake => "uses a channel".into(),
        Builtin::Std(n) => {
            let shown = crate::surface::spoken(n);
            if n.starts_with("print") || n.starts_with("this_will_print") {
                format!("prints (`std::{}`)", shown)
            } else if n.starts_with("input.") {
                format!("reads input (`std::{}`)", shown)
            } else if n.starts_with("time.") {
                format!("waits (`std::{}`)", shown)
            } else if n == "exit" {
                "ends the program (`std::exit`)".into()
            } else if n.starts_with("c.") {
                format!("reaches C (`std::{}`)", shown)
            } else {
                return None;
            }
        }
        _ => return None,
    })
}
