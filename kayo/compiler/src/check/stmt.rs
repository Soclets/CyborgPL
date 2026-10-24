//! Statements.

use crate::ast::{self, Tier};
use crate::diag::{err, Diagnostic, Edit, Result};
use crate::hir::{self, Local, LocalId, LocalKind};
use crate::types::*;

use super::{Checker, FnCtx};

/// Does every path through the block leave (return, fail, break, continue,
/// or an endless loop)?
pub fn leaves_block(b: &hir::Block) -> bool {
    b.stmts.last().map(leaves_stmt).unwrap_or(false)
}

pub fn leaves_stmt(s: &hir::Stmt) -> bool {
    match &s.kind {
        hir::StmtKind::Return(_) | hir::StmtKind::Fail { .. } | hir::StmtKind::Break | hir::StmtKind::Continue => true,
        hir::StmtKind::Block(b) => leaves_block(b),
        hir::StmtKind::If { then, else_: Some(e), .. } => leaves_block(then) && leaves_block(e),
        hir::StmtKind::Match { arms, .. } => arms.iter().all(|a| leaves_block(&a.body)),
        hir::StmtKind::Loop(body) => !has_break(body),
        // A call whose `otherwise` leaves on every path (arms all leaving, or a leaving statement).
        hir::StmtKind::Expr(e) | hir::StmtKind::Let { value: e, .. } => otherwise_leaves(e),
        _ => false,
    }
}

fn otherwise_leaves(e: &hir::Expr) -> bool {
    // `std::exit[…]` leaves; a leaving `otherwise` only leaves on the failure path.
    matches!(&e.kind, hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::Std(n)), .. } if n == "exit")
}

fn has_break(b: &hir::Block) -> bool {
    b.stmts.iter().any(|s| match &s.kind {
        hir::StmtKind::Break => true,
        hir::StmtKind::If { then, else_, .. } => has_break(then) || else_.as_ref().map(has_break).unwrap_or(false),
        hir::StmtKind::Block(b) => has_break(b),
        hir::StmtKind::Match { arms, .. } => arms.iter().any(|a| has_break(&a.body)),
        hir::StmtKind::Expr(e) | hir::StmtKind::Let { value: e, .. } => match &e.kind {
            hir::ExprKind::Call { otherwise: Some(o), .. } | hir::ExprKind::Conv { otherwise: Some(o), .. } => match &**o {
                hir::Otherwise::Leave(s) => matches!(s.kind, hir::StmtKind::Break),
                hir::Otherwise::Arms(arms) => arms.iter().any(|a| has_break(&a.body)),
                _ => false,
            },
            _ => false,
        },
        // a break inside a nested loop belongs to that loop
        _ => false,
    })
}

impl Checker {
    /// A `match` case's or an `otherwise` arm's block, whose last statement
    /// may be its value (owner, 2026-09-29): the statements before it in
    /// the arm's scope, then the value, which sees what they declared.
    pub fn arm_block(&mut self, ctx: &mut FnCtx, b: &ast::Block) -> Result<(hir::Block, Option<hir::Expr>)> {
        let (last, rest) = match b.stmts.split_last() {
            Some((l, r)) => (Some(l), r),
            None => (None, &b.stmts[..]),
        };
        // A nested match that is the case's value says so, `is match …;`
        // (owner, 2026-10-07).
        let value = match last {
            Some(ast::Stmt { kind: ast::StmtKind::Gives { value, .. }, .. }) => value,
            _ => return Ok((self.block(ctx, b)?, None)),
        };
        ctx.push_scope();
        let mut stmts = Vec::new();
        for s in rest {
            stmts.push(self.stmt(ctx, s)?);
        }
        let v = self.expr(ctx, value)?;
        ctx.pop_scope();
        Ok((hir::Block { stmts, span: b.span }, Some(v)))
    }

    /// Which arguments, by position, a call's returned borrow depends on:
    /// its group's, or what its body hands back (checked first if it
    /// waits), or, inside a recursion, every `lent` one.
    pub fn call_deps(&mut self, inst: hir::FnInst, n: usize) -> Result<Vec<usize>> {
        let key = self.instances[inst.0 as usize].key.clone();
        let callee = self.items.fns[key.fn_id.0 as usize].clone();
        let group = callee.ret_lent.as_ref().and_then(|(_, g)| g.clone());
        if let Some(g) = group {
            return Ok(callee.params.iter().enumerate().filter(|(_, p)| p.group.as_ref() == Some(&g)).map(|(i, _)| i).collect());
        }
        if let Some(d) = self.returned_deps(inst)? {
            return Ok(d);
        }
        Ok((0..n).filter(|i| callee.params.get(*i).is_some_and(|p| matches!(p.kind, ParamKind::Lent | ParamKind::LentMut))).collect())
    }

    /// Whether this is a call that hands back a borrow, and whether a
    /// `lent.mut` one.
    pub fn call_lends(&self, e: &hir::Expr) -> Option<bool> {
        let hir::ExprKind::Call { callee: hir::Callee::Fn(inst), .. } = &e.kind else { return None };
        let key = &self.instances[inst.0 as usize].key;
        self.items.fns[key.fn_id.0 as usize].ret_lent.as_ref().map(|(m, _)| *m)
    }

    pub fn block(&mut self, ctx: &mut FnCtx, b: &ast::Block) -> Result<hir::Block> {
        ctx.push_scope();
        let mut stmts = Vec::new();
        for s in &b.stmts {
            stmts.push(self.stmt(ctx, s)?);
        }
        ctx.pop_scope();
        Ok(hir::Block { stmts, span: b.span })
    }

    pub fn stmt(&mut self, ctx: &mut FnCtx, s: &ast::Stmt) -> Result<hir::Stmt> {
        // `check` / `nocheck` in front of a statement covers the statement,
        // and the innermost word wins (`core-features.md`). The tier rides
        // on the statement it covers, so MIR can put it on every site
        // inside — losing it here would make a `nocheck` site checked and a
        // `check` site silent, which is both tiers backwards.
        if let ast::StmtKind::Tier { tier, stmt } = &s.kind {
            // Vouching is a power IEG grants (owner, 2026-10-07).
            match tier {
                Tier::Nocheck => self.power_use(ctx, &["vouch"], None, "vouches with `no_check`".into(), s.span),
                Tier::NoChecks => self.power_use(ctx, &["vouch", "c"], None, "vouches with `NO_FUCKING_CHECKS`".into(), s.span),
                _ => {}
            }
            let saved = ctx.tier;
            ctx.tier = Some(*tier);
            let inner = self.stmt(ctx, stmt)?;
            ctx.tier = saved;
            return Ok(inner);
        }
        if let ast::StmtKind::NoCheat { stmt } = &s.kind {
            let inner = self.stmt(ctx, stmt)?;
            return Ok(hir::Stmt { kind: hir::StmtKind::NoCheat(vec![inner]), span: s.span, tier: ctx.tier });
        }
        let tier = ctx.tier;
        let kind = self.stmt_kind(ctx, s)?;
        Ok(hir::Stmt { kind, span: s.span, tier })
    }

    /// `must …;` (owner, 2026-09-29), lowered to what is there already: an
    /// `if` whose other side reports and ends the test. A comparison's
    /// sides are worked out a second time for the message, as the oracle
    /// has it; `must.fail` catches the call's reasons in a local;
    /// `must.stop` runs its expression in a process of its own.
    fn must(&mut self, ctx: &mut FnCtx, kind: &ast::MustKind, expr: &ast::Expr, span: crate::source::Span) -> Result<hir::StmtKind> {
        use crate::ast::{BinOp, ExprKind as AK, MustKind};
        use crate::types::IntW;
        if !ctx.is_test {
            return err(Diagnostic::error("`must` belongs in a `test fn`: in a program, the prover and `check` say what must hold, before it runs").named("must_misused").at(span));
        }
        let i32_ty = Ty::Int(IntW::Fixed { signed: true, bits: 32 });
        let text = |s: &str| hir::Expr { kind: hir::ExprKind::Str(s.to_string()), ty: Ty::utf8(), span };
        let stmt = |kind: hir::StmtKind| hir::Stmt { kind, span, tier: None };
        let block = |stmts: Vec<hir::Stmt>| hir::Block { stmts, span };
        let builtin = |b: hir::Builtin, args: Vec<hir::Arg>, ty: Ty| hir::Expr { kind: hir::ExprKind::Call { callee: hir::Callee::Builtin(b), args, otherwise: None, fails: Vec::new() }, ty, span };
        let report = |msg: hir::Expr| stmt(hir::StmtKind::Expr(builtin(hir::Builtin::MustReport, vec![hir::Arg::Value(msg)], Ty::Nothing)));
        let fresh = |ctx: &mut FnCtx, name: &str, ty: &Ty| {
            let id = hir::LocalId(ctx.locals.len() as u32);
            ctx.locals.push(Local { name: name.to_string(), ty: ty.clone(), mutability: Some(crate::ast::Mutability::Mut), kind: LocalKind::Owned, from_blank: false, span });
            id
        };
        let read = |id: hir::LocalId, ty: &Ty| hir::Expr { kind: hir::ExprKind::Read(hir::Place { base: id, proj: Vec::new(), ty: ty.clone(), span }), ty: ty.clone(), span };
        let is = |id: hir::LocalId, n: i128| hir::Expr { kind: hir::ExprKind::Binary { op: BinOp::Eq, lhs: Box::new(read(id, &Ty::Int(IntW::Fixed { signed: true, bits: 32 }))), rhs: Box::new(hir::Expr { kind: hir::ExprKind::Int(n), ty: Ty::Int(IntW::Fixed { signed: true, bits: 32 }), span }), op_span: span }, ty: Ty::Bool, span };
        let if_then = |cond: hir::Expr, then: Vec<hir::Stmt>| stmt(hir::StmtKind::If { cond, then: block(then), else_: None });
        match kind {
            MustKind::Hold => {
                let cond = self.expr(ctx, expr)?;
                self.expect_bool(&cond, "`must`'s condition")?;
                // A comparison of values that have a text shows both sides.
                let mut inner = expr;
                while let AK::Paren(e) = &inner.kind {
                    inner = e;
                }
                let printable = |t: &Ty| matches!(t.base(), Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) | Ty::Bool | Ty::Char(_) | Ty::Str(_));
                let msg = match (&inner.kind, &cond.kind) {
                    (AK::Binary { op, op_span, lhs, rhs }, hir::ExprKind::Binary { lhs: hl, rhs: hr, .. }) if op.is_comparison() && printable(&self.resolve(&hl.ty)) && printable(&self.resolve(&hr.ty)) => {
                        let as_text = |e: &ast::Expr| ast::Expr { kind: AK::Conv { target: ast::Type { kind: ast::TypeKind::Path { path: ast::Path { kit: None, segs: vec![ast::Ident { name: "useful".into(), span: e.span }, ast::Ident { name: "str".into(), span: e.span }, ast::Ident { name: "utf8".into(), span: e.span }], span: e.span }, args: Vec::new() }, span: e.span }, expr: Box::new(e.clone()), otherwise: None }, span: e.span };
                        let lit = |s: String| ast::Expr { kind: AK::Str(s), span };
                        let join = |a: ast::Expr, b: ast::Expr| ast::Expr { kind: AK::Binary { op: BinOp::Join, op_span: *op_span, lhs: Box::new(a), rhs: Box::new(b) }, span };
                        let m = join(join(join(lit("`must` does not hold: the left side is ".into()), as_text(lhs)), lit(format!(", and it must be `{}` ", op.spelled()))), as_text(rhs));
                        self.expr(ctx, &m)?
                    }
                    _ => text("`must` does not hold"),
                };
                let not = hir::Expr { kind: hir::ExprKind::Unary { op: crate::ast::UnOp::Not, expr: Box::new(cond) }, ty: Ty::Bool, span };
                Ok(hir::StmtKind::If { cond: not, then: block(vec![report(msg)]), else_: None })
            }
            MustKind::Fail(reason) => {
                let was = ctx.must_fail;
                ctx.must_fail = true;
                let call = self.expr(ctx, expr);
                ctx.must_fail = was;
                let mut call = call?;
                let fails: Vec<String> = match &call.kind {
                    hir::ExprKind::Call { fails, .. } => fails.iter().map(|(n, _)| n.clone()).collect(),
                    hir::ExprKind::Conv { fails, .. } => fails.clone(),
                    _ => Vec::new(),
                };
                if fails.is_empty() {
                    return err(Diagnostic::error("`must.fail` takes something that can fail: a call or a `to.…` with reasons, and this one has none").named("must_misused").at(expr.span));
                }
                let spelled = match reason {
                    Some(r) => format!("must.fail.{}", r.name),
                    None => "must.fail".to_string(),
                };
                let want = match reason {
                    Some(r) => match fails.iter().position(|f| *f == r.name) {
                        Some(i) => Some(i),
                        None => return err(Diagnostic::error(format!("this cannot fail with `{}`: its reasons are {}", r.name, fails.iter().map(|f| format!("`{}`", f)).collect::<Vec<_>>().join(", "))).named("failure_never_happens").at(r.span)),
                    },
                    None => None,
                };
                // Which reason came back: 0 for none, i + 1 for reason i.
                let got = fresh(ctx, "the reason", &i32_ty);
                let arms: Vec<hir::FailArm> = (0..fails.len()).map(|i| hir::FailArm {
                    index: i,
                    bind: None,
                    body: block(vec![stmt(hir::StmtKind::Assign { place: hir::Place { base: got, proj: Vec::new(), ty: i32_ty.clone(), span }, value: hir::Expr { kind: hir::ExprKind::Int(i as i128 + 1), ty: i32_ty.clone(), span } })]),
                    gives: None,
                }).collect();
                match &mut call.kind {
                    hir::ExprKind::Call { otherwise, .. } | hir::ExprKind::Conv { otherwise, .. } => *otherwise = Some(Box::new(hir::Otherwise::Arms(arms))),
                    _ => unreachable!("a call or a conversion"),
                }
                let mut out = vec![
                    stmt(hir::StmtKind::Let { local: got, value: hir::Expr { kind: hir::ExprKind::Int(0), ty: i32_ty.clone(), span }, typed: false }),
                    stmt(hir::StmtKind::Expr(call)),
                    if_then(is(got, 0), vec![report(text(&format!("`{}`: it did not fail", spelled)))]),
                ];
                if let Some(k) = want {
                    for (j, name) in fails.iter().enumerate() {
                        if j != k {
                            out.push(if_then(is(got, j as i128 + 1), vec![report(text(&format!("`{}`: it failed with `{}`", spelled, name)))]));
                        }
                    }
                }
                Ok(hir::StmtKind::Block(block(out)))
            }
            MustKind::Stop => {
                self.items.must_stop_spans.push(expr.span);
                let value = self.expr(ctx, expr)?;
                let k = fresh(ctx, "the stop", &i32_ty);
                Ok(hir::StmtKind::Block(block(vec![
                    stmt(hir::StmtKind::Let { local: k, value: builtin(hir::Builtin::MustStopFork, Vec::new(), i32_ty.clone()), typed: false }),
                    if_then(is(k, 1), vec![stmt(hir::StmtKind::Expr(value)), stmt(hir::StmtKind::Expr(builtin(hir::Builtin::MustStopDone, Vec::new(), Ty::Nothing)))]),
                    if_then(is(k, 2), vec![report(text("`must.stop`: no check fired"))]),
                ])))
            }
        }
    }

    fn stmt_kind(&mut self, ctx: &mut FnCtx, s: &ast::Stmt) -> Result<hir::StmtKind> {
        match &s.kind {
            ast::StmtKind::Must { kind, expr } => self.must(ctx, kind, expr, s.span),
            // A value, or `gives`, anywhere but at the end of a case of a
            // `match` used as one (or of an `otherwise` arm).
            ast::StmtKind::Gives { bare: true, .. } => err(Diagnostic::error("a value alone is not a statement: assign it, or call something with it").named("value_left_unused").at(s.span)),
            ast::StmtKind::Gives { bare: false, .. } => err(Diagnostic::error("`giveS` ends a case of a `match` used as a value, and says that value; this is not the end of one").named("case_value_misplaced").at(s.span)),
            ast::StmtKind::Tier { .. } | ast::StmtKind::NoCheat { .. } => unreachable!("handled in stmt"),
            ast::StmtKind::Const(d) => self.local_const(ctx, d, s.span),
            ast::StmtKind::Let { mutability, name, ty, shadow, value, sealed } => {
                // A const is never shadowed (owner, 2026-10-10).
                if ctx.lookup_const(&name.name).is_some() {
                    return err(Diagnostic::error(format!("'{}' is a const and cannot be shadowed: it is the same value everywhere it is seen", name.name)).named("const_shadowed").at(name.span));
                }
                // Shadowing is never silent. What it may not shadow: a name
                // declared `!$`, and a loop's counter (owner, 2026-09-25) —
                // `immut` only forbids `change`, and a shadow changes nothing.
                let sealed_prev = |ctx: &FnCtx, prev: LocalId| -> Option<Result<hir::StmtKind>> {
                    let p = &ctx.locals[prev.0 as usize];
                    if ctx.sealed.contains(&prev) {
                        return Some(err(Diagnostic::error(format!("'{}' cannot be shadowed: it is `no_shadow`", name.name)).named("shadow_sealed").at(name.span).why_at(p.span, "declared `no_shadow` here")));
                    }
                    if matches!(p.kind, LocalKind::LoopVar | LocalKind::LoopLent) {
                        return Some(err(Diagnostic::error(format!("'{}' belongs to the loop and cannot be shadowed: the header alone says what it is on every pass", name.name)).named("counter_shadowed").at(name.span).why_at(p.span, "the loop's counter")));
                    }
                    None
                };
                if let Some(prev) = ctx.in_current_scope(&name.name) {
                    let p = &ctx.locals[prev.0 as usize];
                    if *shadow {
                        if let Some(e) = sealed_prev(ctx, prev) {
                            return e;
                        }
                    } else {
                        return err(Diagnostic::error(format!("'{n}' is already declared in this scope; to shadow it deliberately write `shadow ${n} := …;`", n = name.name)).named("already_declared").at(name.span).fix_edit(name.span, Edit::Swap(vec!["let ".into()]), "shadow ").why_at(p.span, "the first '".to_string() + &name.name + "' is declared here"));
                    }
                } else if let Some(prev) = ctx.visible_outside(&name.name) {
                    let p = &ctx.locals[prev.0 as usize];
                    if *shadow {
                        if let Some(e) = sealed_prev(ctx, prev) {
                            return e;
                        }
                    } else {
                        return err(Diagnostic::error(format!("'{n}' is already declared in an enclosing scope; to shadow it deliberately write `shadow ${n} := …;`", n = name.name)).named("shadow_unsaid").at(name.span).fix_edit(name.span, Edit::Swap(vec!["let ".into()]), "shadow ").why_at(p.span, "the first '".to_string() + &name.name + "' is declared here"));
                    }
                } else if *shadow {
                    if self.const_visible(ctx, &name.name) {
                        return err(Diagnostic::error(format!("'{}' is a const and cannot be shadowed", name.name)).named("const_shadowed").at(name.span));
                    }
                    return err(Diagnostic::error(format!("`shadow` takes a visible '{n}', and none is visible here: write `let ${n} := …;`", n = name.name)).named("nothing_to_shadow").at(name.span).fix_edit(name.span, Edit::Swap(vec!["shadow ".into()]), "let "));
                }
                if !*shadow && self.const_visible(ctx, &name.name) {
                    return err(Diagnostic::error(format!("'{}' is a const of this kit and cannot be redeclared", name.name)).named("const_shadowed").at(name.span));
                }
                let declared = match ty {
                    Some(t) => {
                        let rt = self.body_type(ctx, t)?;
                        Some(rt)
                    }
                    None => None,
                };
                if let Some(t) = &declared {
                    if let Ty::Ptr { .. } = t.base() {
                        return err(Diagnostic::error(format!("`{}` exists only at the boundary: in an `extern fn` signature or a `layout.C` field — a declaration cannot make a pointer; a pointer comes only from a lend or from C", self.spell(t))).named("pointer_outside_boundary").at(ty.as_ref().unwrap().span));
                    }
                }
                // A borrow a function hands back is held by a new name
                // (owner, 2026-09-29), which borrows what the call lent.
                let saved = ctx.lend_call_ok.take();
                if let ast::ExprKind::Call { .. } = &value.kind {
                    ctx.lend_call_ok = Some(value.span);
                }
                // `let 'f': fn[…] = op +;` takes its type from the declaration.
                self.store_hint = declared.as_ref().map(|t| (self.resolve(t), value.span));
                self.want = declared.as_ref().map(|t| self.resolve(t));
                let was_plain = self.plain_numbers;
                self.plain_numbers = declared.as_ref().is_some_and(|t| crate::check::expr::plain_number(&self.resolve(t)));
                let v = match (&value.kind, declared.as_ref().map(|t| self.resolve(t))) {
                    (ast::ExprKind::OpValue { op, spelled, unary }, Some(Ty::Fn(ft))) => self.op_value(ctx, op, spelled, *unary, &ft, value.span),
                    _ => self.expr_top(ctx, value, true),
                };
                ctx.lend_call_ok = saved;
                self.store_hint = None;
                self.want = None;
                self.plain_numbers = was_plain;
                let v = v?;
                let lends = self.call_lends(&v);
                if let hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::ListAppend), .. } = &v.kind {
                    return err(Diagnostic::error("`list.append` hands nothing back and cannot be used as a value: it changes the list in place").named("call_gives_nothing").at(value.span));
                }
                if let (hir::ExprKind::List(items), None) = (&v.kind, &declared) {
                    if items.is_empty() {
                        return err(Diagnostic::error(format!("`[]` says nothing about what '{}' holds, and nothing on the left says \"list\": write `list.empty` (the prover picks the element type from what is appended later), or give it a type", name.name)).named("empty_untyped").at(value.span).fix_edit(value.span, Edit::Replace, "list.empty"));
                    }
                }
                let (v, local_ty, kind) = match &declared {
                    Some(t) => {
                        if let (hir::ExprKind::List(items), Ty::Seq { kind: SeqKind::Array, .. }) = (&v.kind, self.resolve(t).base()) {
                            if items.is_empty() {
                                return err(Diagnostic::error(format!("an empty `array` could never be filled: an array is fixed at the size it was made with — make '{}' a `list`, or give it its elements", name.name)).named("array_never_filled").at(value.span).fix_edit(ty.as_ref().map_or(value.span, |w| w.span), Edit::Swap(vec!["array".into()]), "list"));
                            }
                        }
                        // Or the declared type is the value's: `let $n: d64 = …`.
                        let value_ty = crate::check::expr::conv_spelling(&self.spell(&self.resolve(&v.ty)));
                        // A whole number too big for the type written: the
                        // narrowest of the same sign that holds it.
                        let wider = match (&value.kind, self.resolve(t).base()) {
                            (ast::ExprKind::Int(text), Ty::Int(crate::types::IntW::Fixed { signed, .. })) => text.replace('_', "").parse::<i128>().ok().and_then(|n| {
                                [8u32, 16, 32, 64, 128].into_iter().find(|b| {
                                    let (lo, hi) = crate::bigint::type_range(*signed, *b);
                                    lo.to_i128().is_some_and(|lo| lo <= n) && hi.to_i128().is_some_and(|hi| n <= hi)
                                }).map(|b| format!("{}{}", if *signed { "i" } else { "u" }, b))
                            }),
                            _ => None,
                        };
                        let v = self.expect_store(ctx, v, t, "the declared type").map_err(|d| match (d.name, ty) {
                            (Some("wrong_type" | "no_implicit_conversion"), Some(written)) if !value_ty.contains(' ') => Box::new(d.fix_edit(written.span, crate::diag::Edit::Replace, value_ty.clone())),
                            (Some("value_does_not_fit"), Some(written)) if wider.is_some() => Box::new(d.fix_edit(written.span, crate::diag::Edit::Replace, wider.clone().unwrap())),
                            _ => d,
                        })?;
                        if lends.is_none() {
                            self.check_stored_value(ctx, &v, t)?;
                        }
                        let kind = match lends {
                            Some(true) => LocalKind::LentMut,
                            Some(false) => LocalKind::Lent,
                            None => LocalKind::Owned,
                        };
                        (v, t.clone(), kind)
                    }
                    None => {
                        // `ptr.null` alone says nothing of what it points at
                        // (owner, 2026-10-21: "widened (a)"), nor a list of
                        // nothing else.
                        let null = |e: &ast::Expr| matches!(&e.kind, ast::ExprKind::Path(p) if p.is(&["ptr", "null"]));
                        if null(value) {
                            return err(Diagnostic::error(format!("`ptr.null` alone does not say what it points at: give '{}' its type, `let ${}: ptr[u8] = ptr.null;`", name.name, name.name)).named("empty_untyped").at(value.span));
                        }
                        if let ast::ExprKind::List(items) = &value.kind {
                            if !items.is_empty() && items.iter().all(null) {
                                return err(Diagnostic::error(format!("`ptr.null` alone does not say what it points at, so '{}' cannot hold it: give it its type, `let ${}: list[ptr[u8]] = …;`", name.name, name.name)).named("empty_untyped").at(value.span));
                            }
                        }
                        let mut v = v;
                        crate::check::expr::settle_exact(&mut v)?;
                        let vt = self.resolve(&v.ty);
                        if vt == Ty::Nothing {
                            return err(Diagnostic::error(format!("'{}' would hold nothing: the value on the right returns nothing", name.name)).named("call_gives_nothing").at(value.span));
                        }
                        let kind = match (&v.kind, lends) {
                            (hir::ExprKind::Lend { mutable, .. }, _) => if *mutable { LocalKind::LentMut } else { LocalKind::Lent },
                            (_, Some(true)) => LocalKind::LentMut,
                            (_, Some(false)) => LocalKind::Lent,
                            _ => LocalKind::Owned,
                        };
                        if lends.is_none() {
                            self.check_stored_value(ctx, &v, &vt)?;
                        }
                        (v, self.infer_local_type(&vt), kind)
                    }
                };
                let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: local_ty, mutability: *mutability, kind, from_blank: false, span: name.span });
                // A `:=` whole number keeps the range its operands allow.
                if declared.is_none() {
                    let t = self.resolve(&v.ty);
                    if matches!(t.base(), Ty::Int(IntW::Free(_))) || t.is_exact() {
                        if let Some(r) = self.int_span(ctx, &v) {
                            ctx.int_ranges.insert(id, r);
                        }
                    }
                }
                if *mutability == Some(ast::Mutability::Mut) {
                    ctx.promised.push(id);
                } else if let Some(text) = crate::check::expr::literal_written(value) {
                    // Never changed, so its value is the literal as typed.
                    ctx.written.insert(id, text);
                }
                if *sealed {
                    ctx.sealed.insert(id);
                }
                // A borrowing name remembers what it borrows.
                if matches!(kind, LocalKind::Lent | LocalKind::LentMut) {
                    let direct: Vec<LocalId> = match &v.kind {
                        hir::ExprKind::Lend { place, .. } => vec![place.base],
                        hir::ExprKind::Call { callee: hir::Callee::Fn(inst), args, .. } => {
                            let deps = self.call_deps(*inst, args.len())?;
                            args.iter().enumerate().filter(|(i, _)| deps.contains(i)).filter_map(|(_, a)| match a {
                                hir::Arg::Lend { place, .. } => Some(place.base),
                                _ => None,
                            }).collect()
                        }
                        _ => Vec::new(),
                    };
                    let mut all = Vec::new();
                    for d in direct {
                        match ctx.borrow_src.get(&d) {
                            Some(s) => all.extend(s.iter().copied()),
                            None => all.push(d),
                        }
                    }
                    ctx.borrow_src.insert(id, all);
                }
                match &v.kind {
                    hir::ExprKind::List(items) => {
                        ctx.literal_lengths.insert(id, items.len());
                    }
                    hir::ExprKind::ListEmpty => {
                        ctx.literal_lengths.insert(id, 0);
                    }
                    _ => {}
                }
                Ok(hir::StmtKind::Let { local: id, value: v, typed: declared.is_some() })
            }
            ast::StmtKind::Assign { target, value } => {
                let place = self.place(ctx, target, PlaceUse::Write)?;
                self.store_hint = Some((self.resolve(&place.ty), value.span));
                let v = self.expr_top(ctx, value, true);
                self.store_hint = None;
                let v = v?;
                let pt = self.resolve(&place.ty);
                let what = self.place_name(ctx, &place);
                let v = self.expect_store(ctx, v, &pt, &what)?;
                self.check_stored_value(ctx, &v, &pt)?;
                Ok(hir::StmtKind::Assign { place, value: v })
            }
            ast::StmtKind::Expr(e) => {
                let v = self.expr_stmt_value(ctx, e)?;
                Ok(hir::StmtKind::Expr(v))
            }
            ast::StmtKind::If { cond, then, else_ } => {
                let c = self.expr(ctx, cond)?;
                self.expect_bool(&c, "the condition of an `if`")?;
                let t = self.block(ctx, then)?;
                let e = match else_ {
                    Some(ast::ElseArm::Block(b)) => Some(self.block(ctx, b)?),
                    Some(ast::ElseArm::If(s)) => {
                        let st = self.stmt(ctx, s)?;
                        Some(hir::Block { span: st.span, stmts: vec![st] })
                    }
                    None => None,
                };
                Ok(hir::StmtKind::If { cond: c, then: t, else_: e })
            }
            ast::StmtKind::Loop(body) => {
                ctx.loop_depth += 1;
                let b = self.block(ctx, body)?;
                ctx.loop_depth -= 1;
                Ok(hir::StmtKind::Loop(b))
            }
            ast::StmtKind::While { cond, body } => {
                let c = self.expr(ctx, cond)?;
                self.expect_bool(&c, "the condition of a `while`")?;
                ctx.loop_depth += 1;
                let b = self.block(ctx, body)?;
                ctx.loop_depth -= 1;
                Ok(hir::StmtKind::While { cond: c, body: b })
            }
            ast::StmtKind::For { nocheat, seq, para, perm, name, pos, ty, iter, body } => {
                let mut s = self.for_stmt(ctx, *seq, *para, *perm, name, pos.as_ref(), ty.as_ref(), iter, body)?;
                if let hir::StmtKind::ForRange { nocheat: n, .. } | hir::StmtKind::ForEach { nocheat: n, .. } = &mut s {
                    *n = *nocheat;
                }
                Ok(s)
            }
            ast::StmtKind::Return(value) => {
                if ctx.is_task {
                    return err(Diagnostic::error("a task is a block, not a function: `return` cannot leave it").named("mcps::task_returns").at(s.span));
                }
                if ctx.is_main {
                    return err(Diagnostic::error("`MAIN` is a block, not a function: it has no `return` — ending it exits 0, and any other code is `std::exit(n)`").named("main_form").at(s.span));
                }
                match value {
                    Some(v) => {
                        if ctx.ret == Ty::Nothing {
                            return err(Diagnostic::error(format!("`{}` returns nothing, so `return` takes no value: write `return;` or give the function a `-> type`", ctx.name)).named("return_type_missing").at(v.span));
                        }
                        // A borrow another call hands back, returned by a
                        // function that does not say it hands one back.
                        if let (None, ast::ExprKind::Call { path, .. }) = (&ctx.ret_lent, &v.kind) {
                            if let Ok(crate::resolve::ItemRef::Fn(id)) = self.items.lookup_path(path, ctx.module) {
                                if self.items.fns[id.0 as usize].ret_lent.is_some() {
                                    return err(Diagnostic::error(format!("`{}` returns a borrow, so its signature must say so: `-> lent {}` — `{}` hands one back", ctx.name, self.spell(&ctx.ret), self.items.fns[id.0 as usize].name)).named("returned_borrow_unsaid").at(v.span));
                                }
                            }
                        }
                        // A borrow handed back (owner, 2026-09-29): of a
                        // `lent` parameter, in the group the return names.
                        if let ast::ExprKind::Verb { verb: verb @ (ast::Verb::Lend | ast::Verb::LendMut), expr, .. } = &v.kind {
                            let Some((mutable, group)) = ctx.ret_lent.clone() else {
                                return err(Diagnostic::error(format!("`{}` returns a borrow, so its signature must say so: `-> lent {}`", ctx.name, self.spell(&ctx.ret))).named("returned_borrow_unsaid").at(v.span));
                            };
                            let lend_mut = *verb == ast::Verb::LendMut;
                            let place = self.place(ctx, expr, if lend_mut { PlaceUse::LendMut } else { PlaceUse::Lend })?;
                            // A local borrow is a borrow of what it borrows.
                            let owners: Vec<LocalId> = ctx.borrow_src.get(&place.base).cloned().unwrap_or_else(|| vec![place.base]);
                            for owner in owners {
                                let who = ctx.locals[owner.0 as usize].name.clone();
                                let Some((pi, pg)) = ctx.lent_params.get(&owner).cloned() else {
                                    return err(Diagnostic::error(format!("`{}` hands back a borrow of '{}', which dies when it returns: only a borrow of a `lent` parameter outlives the call", ctx.name, who)).named("borrow_outlives_owner").at(v.span));
                                };
                                if let Some(g) = &group {
                                    if pg.as_ref() != Some(g) {
                                        let is = match &pg {
                                            Some(p) => format!("'{}' is `lent@{}`", who, p),
                                            None => format!("'{}' is in no group", who),
                                        };
                                        return err(Diagnostic::error(format!("`{}` returns a borrow of '{}', which is not in @{}: {}", ctx.name, who, g, is)).named("borrow_outside_group").at(v.span));
                                    }
                                }
                                ctx.returned.insert(pi);
                            }
                            if mutable && !lend_mut {
                                return err(Diagnostic::error(format!("`{}` hands back `lent.mut`: `return lend.mut ${};`", ctx.name, ctx.locals[place.base.0 as usize].name)).named("returned_borrow_form").at(v.span).fix_edit(v.span, Edit::Verb, "lend.mut"));
                            }
                            if !mutable && lend_mut {
                                return err(Diagnostic::error(format!("`{}` hands back a plain `lent`: `return lend ${};`, or declare `-> lent.mut …`", ctx.name, ctx.locals[place.base.0 as usize].name)).named("returned_borrow_form").at(v.span).fix_edit(v.span, Edit::Verb, "lend"));
                            }
                            let ty = place.ty.clone();
                            return Ok(hir::StmtKind::Return(Some(hir::Expr { kind: hir::ExprKind::Lend { place, mutable: lend_mut }, ty, span: v.span })));
                        }
                        // Or the borrow another call hands back, when what it
                        // borrows is this function's own `lent` parameters.
                        if let (Some((mutable, group)), ast::ExprKind::Call { .. }) = (ctx.ret_lent.clone(), &v.kind) {
                            let saved = ctx.lend_call_ok.replace(v.span);
                            let e = self.expr_top(ctx, v, true);
                            ctx.lend_call_ok = saved;
                            let e = e?;
                            if let (Some(m), hir::ExprKind::Call { callee: hir::Callee::Fn(inst), args, .. }) = (self.call_lends(&e), &e.kind) {
                                if m != mutable {
                                    return err(Diagnostic::error(format!("`{}` hands back a {} borrow, and this call hands back a {} one", ctx.name, if mutable { "`lent.mut`" } else { "plain `lent`" }, if m { "`lent.mut`" } else { "plain `lent`" })).named("returned_borrow_form").at(v.span));
                                }
                                let key = self.instances[inst.0 as usize].key.clone();
                                let callee = self.items.fns[key.fn_id.0 as usize].clone();
                                let _ = callee;
                                // With no group written, what the callee's
                                // body hands back (check claude, 2026-09-29).
                                let deps = self.call_deps(*inst, args.len())?;
                                for (i, a) in args.iter().enumerate() {
                                    if !deps.contains(&i) {
                                        continue;
                                    }
                                    let hir::Arg::Lend { place, .. } = a else { continue };
                                    let owners: Vec<LocalId> = ctx.borrow_src.get(&place.base).cloned().unwrap_or_else(|| vec![place.base]);
                                    for owner in owners {
                                        let who = ctx.locals[owner.0 as usize].name.clone();
                                        let Some((pi, pg)) = ctx.lent_params.get(&owner).cloned() else {
                                            return err(Diagnostic::error(format!("`{}` hands back a borrow of '{}', which dies when it returns: only a borrow of a `lent` parameter outlives the call", ctx.name, who)).named("borrow_outlives_owner").at(v.span));
                                        };
                                        if let Some(g) = &group {
                                            if pg.as_ref() != Some(g) {
                                                return err(Diagnostic::error(format!("`{}` returns a borrow of '{}', which is not in @{}", ctx.name, who, g)).named("borrow_outside_group").at(v.span));
                                            }
                                        }
                                        ctx.returned.insert(pi);
                                    }
                                }
                                return Ok(hir::StmtKind::Return(Some(e)));
                            }
                            return err(Diagnostic::error(format!("`{}` hands back a borrow: `return lend $a;`, of one of its `lent` parameters", ctx.name)).named("returned_borrow_form").at(v.span));
                        }
                        if ctx.ret_lent.is_some() {
                            return err(Diagnostic::error(format!("`{}` hands back a borrow: `return lend $a;`, of one of its `lent` parameters", ctx.name)).named("returned_borrow_form").at(v.span));
                        }
                        let e = self.expr_top(ctx, v, true)?;
                        let rt = ctx.ret.clone();
                        let e = self.expect_store(ctx, e, &rt, "the return type")?;
                        self.check_stored_value(ctx, &e, &rt)?;
                        Ok(hir::StmtKind::Return(Some(e)))
                    }
                    None => {
                        if ctx.ret != Ty::Nothing {
                            return err(Diagnostic::error(format!("`{}` returns {}: `return` needs the value", ctx.name, crate::types::a_type(&self.spell(&ctx.ret)))).named("missing_return").at(s.span));
                        }
                        Ok(hir::StmtKind::Return(None))
                    }
                }
            }
            ast::StmtKind::Fail { name, payload } => self.fail_stmt(ctx, name, payload.as_ref(), s.span),
            ast::StmtKind::Break => {
                if ctx.loop_depth == 0 {
                    return err(Diagnostic::error("`break` outside a loop: there is nothing to leave").named("outside_loop").at(s.span));
                }
                Ok(hir::StmtKind::Break)
            }
            ast::StmtKind::Continue => {
                if ctx.loop_depth == 0 {
                    return err(Diagnostic::error("`continue` outside a loop: there is nothing to continue").named("outside_loop").at(s.span));
                }
                Ok(hir::StmtKind::Continue)
            }
            ast::StmtKind::Block(b) => Ok(hir::StmtKind::Block(self.block(ctx, b)?)),
            ast::StmtKind::Match { value, arms } => self.match_stmt(ctx, value, arms),
            ast::StmtKind::Concur { detached, captures, body, otherwise } => self.concur_stmt(ctx, *detached, captures, body, otherwise.as_ref(), s.span),
            ast::StmtKind::Whichever { arms, otherwise } => self.whichever_stmt(ctx, arms, otherwise.as_ref(), s.span),
            ast::StmtKind::InlineAsm { text, bindings } => {
                self.power_use(ctx, &["inline_assembly"], None, "runs assembly (`inline.ASM`)".into(), s.span);
                if self.needs_mark(ctx) {
                    self.power_first()?;
                    return err(Diagnostic::error("an inline block is a no_check site: nothing inside it can be proven — write `no_check inline.ASM { … }`").named("no_check_missing").at(s.span).fix_edit(s.span, Edit::Before, "no_check "));
                }
                let mut ins = Vec::new();
                let mut outs = Vec::new();
                let mut lends = Vec::new();
                let mut clobbers = Vec::new();
                for b in bindings {
                    match b {
                        ast::AsmBinding::Clobber(r) => clobbers.push(r.name.clone()),
                        ast::AsmBinding::Lend { name, mutable, reg } => {
                            let Some(id) = ctx.lookup(&name.name) else {
                                return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared", name.name)).named("not_declared").at(name.span), &name.name, name.span));
                            };
                            if *mutable {
                                let local = &ctx.locals[id.0 as usize];
                                let why = if local.mutability == Some(ast::Mutability::Immut) {
                                    Some("is `immut`")
                                } else if local.kind == LocalKind::Lent {
                                    Some("is lent (shared)")
                                } else if matches!(local.kind, LocalKind::LoopVar | LocalKind::LoopLent) {
                                    Some("is the loop's element")
                                } else {
                                    None
                                };
                                if let Some(why) = why {
                                    return err(Diagnostic::error(format!("'{}' {}, so `lend.mut ${}` cannot hand the assembly its address to write through", name.name, why, name.name)).named(if why.starts_with("is lent") { "changed_through_lend" } else { "cannot_change_immut" }).at(name.span));
                                }
                            }
                            if *mutable {
                                ctx.changed.insert(id);
                            }
                            lends.push((id, *mutable, reg.name.clone()));
                        }
                        ast::AsmBinding::Reg { name, dir, reg } => {
                            match dir {
                                ast::AsmDir::In => {
                                    let Some(id) = ctx.lookup(&name.name) else {
                                        return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared", name.name)).named("not_declared").at(name.span), &name.name, name.span));
                                    };
                                    ins.push((id, reg.name.clone()));
                                }
                                ast::AsmDir::Out => {
                                    // declares the variable, like a `let`
                                    let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: Ty::Int(IntW::Free(InferId(0))), mutability: None, kind: LocalKind::Owned, from_blank: false, span: name.span });
                                    outs.push((id, reg.name.clone()));
                                }
                                ast::AsmDir::InOut => {
                                    let Some(id) = ctx.lookup(&name.name) else {
                                        return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared", name.name)).named("not_declared").at(name.span), &name.name, name.span));
                                    };
                                    ins.push((id, reg.name.clone()));
                                    outs.push((id, reg.name.clone()));
                                    ctx.changed.insert(id);
                                }
                            }
                        }
                    }
                }
                Ok(hir::StmtKind::InlineAsm { text: text.clone(), ins, outs, lends, clobbers })
            }
        }
    }

    fn const_visible(&self, ctx: &FnCtx, name: &str) -> bool {
        self.items.modules[ctx.module.0 as usize].const_named(name).is_some()
    }

    /// The type a `:=` local takes from its value: a fixed source keeps its
    /// type; a free value stays free for Precog to size.
    fn infer_local_type(&self, vt: &Ty) -> Ty {
        vt.clone()
    }

    pub fn expect_bool(&self, e: &hir::Expr, what: &str) -> Result<()> {
        let t = self.resolve(&e.ty);
        if let Ty::Simd { elem, lanes } = t.base() {
            let what_it_is = if elem.base() == &Ty::Bool { "one answer per lane".to_string() } else { format!("{} lanes of `{}`", lanes, self.spell(elem)) };
            return err(Diagnostic::error(format!("{} must be a `bool`, and this is {}: a vector is never a single answer — it is {}", what, crate::types::a_type(&self.spell(&t)), what_it_is)).named("no_truthiness")
                .at(e.span)
                .fix_edit(e.span, crate::diag::Edit::Wrap, "simd.all")
                .fix_edit(e.span, crate::diag::Edit::Wrap, "simd.any"));
        }
        if t.base() != &Ty::Bool {
            return err(Diagnostic::error(format!("{} must be a `bool`, and this is {}: there is no truthiness — compare it (`$n not_equal 0`)", what, crate::types::a_type(&self.spell(&t)))).named("no_truthiness").at(e.span));
        }
        Ok(())
    }

    fn fail_stmt(&mut self, ctx: &mut FnCtx, name: &ast::Ident, payload: Option<&ast::Expr>, span: Span) -> Result<hir::StmtKind> {
        if ctx.is_main {
            return err(Diagnostic::error("`MAIN` cannot fail: it exits — `std::exit(1)` says the code").named("main_cannot_fail").at(span));
        }
        let Some(index) = ctx.fails.iter().position(|(n, _)| *n == name.name) else {
            if ctx.fails.is_empty() {
                return err(Diagnostic::error(format!("`{}` cannot fail: its signature has no `fail[…]` — add `-> … or fail[{}]` to say it can", ctx.name, name.name)).named("failure_not_listed").at(name.span));
            }
            return err(Diagnostic::error(format!("`{}` does not list `{}` among its failures: its signature says fail[{}]", ctx.name, name.name, ctx.fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", "))).named("failure_not_listed").at(name.span));
        };
        let expected = ctx.fails[index].1.clone();
        let payload = match (payload, expected) {
            (None, None) => None,
            (Some(p), Some(t)) => {
                let e = self.expr(ctx, p)?;
                let e = self.expect_store(ctx, e, &t, "the failure's payload")?;
                self.check_stored_value(ctx, &e, &t)?;
                Some(e)
            }
            (None, Some(t)) => return err(Diagnostic::error(format!("`{}` carries {}, and `fail {}` needs it: `fail {}[…]`", name.name, crate::types::a_type(&self.spell(&t)), name.name, name.name)).named("data_missing").at(span)),
            (Some(p), None) => return err(Diagnostic::error(format!("`{}` carries nothing: write `fail {};` with no value", name.name, name.name)).named("carries_nothing").at(p.span)),
        };
        Ok(hir::StmtKind::Fail { name: name.name.clone(), index, payload })
    }

    #[allow(clippy::too_many_arguments)]
    fn for_stmt(&mut self, ctx: &mut FnCtx, seq: bool, para: bool, perm: bool, name: &ast::QName, pos: Option<&ast::QName>, ty: Option<&ast::Type>, iter: &ast::ForIter, body: &ast::Block) -> Result<hir::StmtKind> {
        if let Some(prev) = ctx.lookup(&name.name) {
            let p = &ctx.locals[prev.0 as usize];
            return err(Diagnostic::error(format!("'{}' is already declared; a loop variable is a new name (shadowing is never silent)", name.name)).named("already_declared").at(name.span).why_at(p.span, "declared here"));
        }
        match iter {
            ast::ForIter::Range { start, stop, step, exclusive } => {
                let counter_ty = match ty {
                    Some(t) => {
                        let rt = self.body_type(ctx, t)?;
                        if !rt.is_int() {
                            return err(Diagnostic::error(format!("a range counter is an integer; `{}` is not", self.spell(&rt))).named("needs_integer").at(t.span));
                        }
                        rt
                    }
                    None => Ty::Int(IntW::Free(InferId(0))),
                };
                let s = self.expr(ctx, start)?;
                let s = self.expect_store(ctx, s, &counter_ty, "the counter")?;
                let e = self.expr(ctx, stop)?;
                // A `before` end is never the counter's value, so it need not
                // fit the counter's type (owner, 2026-10-12): it is held in
                // 128 bits, as the step's test is.
                let e = if *exclusive { self.expect_store(ctx, e, &Ty::int(true, 128), "the end")? } else { self.expect_store(ctx, e, &counter_ty, "the counter")? };
                let st = match step {
                    Some(st) => {
                        let v = self.expr(ctx, st)?;
                        // "The step is not one of those values" (owner,
                        // 2026-09-23): it is its own signed number, an
                        // instruction to go up or down, so it is checked as
                        // an `i64`, never as the counter's type — `-1` is a
                        // fine step for a `u64` counter.
                        let v = self.expect_store(ctx, v, &crate::types::Ty::int(true, 64), "the step")?;
                        if let hir::ExprKind::Int(0) = v.kind {
                            return err(Diagnostic::error("a step of 0 would never end: the counter must move").named("step_zero").at(st.span).fix_edit(st.span, Edit::Replace, "1"));
                        }
                        Some(v)
                    }
                    None => None,
                };
                // The counter lives inside the loop, or after it with `perm`.
                let outer_scope_len = ctx.scopes.len();
                if !perm {
                    ctx.push_scope();
                }
                let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: counter_ty, mutability: None, kind: LocalKind::LoopVar, from_blank: false, span: name.span });
                ctx.loop_depth += 1;
                let b = self.block(ctx, body)?;
                ctx.loop_depth -= 1;
                if !perm {
                    ctx.pop_scope();
                }
                debug_assert!(ctx.scopes.len() == outer_scope_len);
                Ok(hir::StmtKind::ForRange { counter: id, start: s, stop: e, step: st, exclusive: *exclusive, body: b, perm, seq, para, nocheat: false })
            }
            ast::ForIter::Over(e) => {
                if ty.is_some() {
                    return err(Diagnostic::error("a list loop's variable takes no type: each element's type is the list's").named("loop_variable_typed").at(name.span).fix_edit(crate::source::Span::new(name.span.file, name.span.end as usize, ty.as_ref().map_or(name.span.end, |t| t.span.end) as usize), Edit::Replace, ""));
                }
                let it = self.expr(ctx, e)?;
                let it_ty = self.resolve(&it.ty);
                if matches!(it_ty.base(), Ty::Struct { .. } | Ty::Enum { .. }) && matches!(it.kind, hir::ExprKind::Lend { .. } | hir::ExprKind::Give(_)) {
                    return self.for_next(ctx, perm, name, pos, e, it, &it_ty, body);
                }
                let (elem_ty, elem_kind) = match &it.kind {
                    hir::ExprKind::Lend { mutable, .. } => {
                        let elem = self.elem_type_of(&it_ty, e.span, "a `for` over")?;
                        (elem, if *mutable { LocalKind::LentMut } else { LocalKind::Lent })
                    }
                    hir::ExprKind::Give(_) => {
                        let elem = self.elem_type_of(&it_ty, e.span, "a `for` over")?;
                        (elem, LocalKind::Owned)
                    }
                    hir::ExprKind::Call { callee: hir::Callee::Builtin(b), .. } => match b {
                        hir::Builtin::StrEach(_) | hir::Builtin::MapKeys | hir::Builtin::MapValues | hir::Builtin::MapPairs => {
                            let elem = self.elem_type_of(&it_ty, e.span, "a `for` over")?;
                            (elem, LocalKind::Lent)
                        }
                        _ => return err(Diagnostic::error("a `for` runs over a list (`lend $xs` / `give $xs`), a `range[…]`, a text walk (`str.each.grapheme(…)`) or a map walk (`map.keys(…)`)").named("not_walkable").at(e.span)),
                    },
                    hir::ExprKind::Local(_) | hir::ExprKind::Read(_) if it_ty.is_seq() || matches!(it_ty.base(), Ty::Bundle(_)) => {
                        return err(Diagnostic::error(format!("say how the loop takes the list: `for ${} in lend $xs` looks at each element, `give $xs` consumes the list", name.name)).named("owned_value_bare").at(e.span));
                    }
                    _ if it_ty.is_text() => {
                        return err(Diagnostic::error("a text is not a list of anything until you say what: `str.each.grapheme(lend $s)`, `.point`, or `.byte`").named("text_unit_unsaid").at(e.span));
                    }
                    _ => return err(Diagnostic::error(format!("a `for` needs a list to run over, and this is {}", crate::types::a_type(&self.spell(&it_ty)))).named("not_walkable").at(e.span)),
                };
                // A mixed bundle: the body is checked once per element, unrolled.
                if let Ty::Bundle(ts) = self.resolve(&elem_ty) {
                    if perm {
                        return err(Diagnostic::error("`perm` on a loop over a mixed bundle: the elements have different types, so there is no one variable to keep").named("bundle_loop_form").at(name.span));
                    }
                    let place = match &it.kind {
                        hir::ExprKind::Lend { place, .. } => place.clone(),
                        _ => return err(Diagnostic::error("a mixed bundle is walked with `lend`").named("bundle_loop_form").at(e.span)),
                    };
                    let mut stmts = Vec::new();
                    for (i, t) in ts.iter().enumerate() {
                        ctx.push_scope();
                        let mut pos_let = None;
                        if let Some(p) = pos {
                            let pid = ctx.declare(&p.name, Local { name: p.name.clone(), ty: Ty::u64(), mutability: None, kind: LocalKind::LoopVar, from_blank: false, span: p.span });
                            pos_let = Some(hir::Stmt { kind: hir::StmtKind::Let { local: pid, value: hir::Expr { kind: hir::ExprKind::Int(i as i128 + 1), ty: Ty::u64(), span: p.span }, typed: true }, span: p.span, tier: ctx.tier });
                        }
                        let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: t.clone(), mutability: None, kind: LocalKind::LoopVar, from_blank: false, span: name.span });
                        let value = hir::Expr { kind: hir::ExprKind::BundleElem { place: place.clone(), index: i }, ty: t.clone(), span: e.span };
                        let saved_depth = ctx.loop_depth;
                        ctx.loop_depth = 0; // break/continue are not offered inside an unrolled walk
                        let b = self.block(ctx, body)?;
                        ctx.loop_depth = saved_depth;
                        ctx.pop_scope();
                        let mut inner: Vec<hir::Stmt> = pos_let.into_iter().collect();
                        inner.push(hir::Stmt { kind: hir::StmtKind::Let { local: id, value, typed: false }, span: name.span, tier: ctx.tier });
                        inner.push(hir::Stmt { kind: hir::StmtKind::Block(b), span: body.span, tier: ctx.tier });
                        stmts.push(hir::Stmt { kind: hir::StmtKind::Block(hir::Block { stmts: inner, span: body.span }), span: body.span, tier: ctx.tier });
                    }
                    return Ok(hir::StmtKind::Block(hir::Block { stmts, span: body.span }));
                }
                if !perm {
                    ctx.push_scope();
                }
                // The position is a `u64` from 1, like every position.
                let pos_id = pos.map(|p| ctx.declare(&p.name, Local { name: p.name.clone(), ty: Ty::u64(), mutability: None, kind: LocalKind::LoopVar, from_blank: false, span: p.span }));
                let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: elem_ty, mutability: None, kind: if elem_kind == LocalKind::Owned { LocalKind::LoopVar } else { elem_kind }, from_blank: false, span: name.span });
                // A lent element is a loop variable too: not assignable.
                // One taken by `lend` stays a borrow (examples/gradebook: a
                // `for 's' in lend 'students'` gave each Student away and
                // dropped it, and the list dropped them again).
                // A plain number is read as itself.
                let owns = self.resolve(&ctx.locals[id.0 as usize].ty).is_owned();
                ctx.locals[id.0 as usize].kind = if elem_kind == LocalKind::Lent && owns { LocalKind::LoopLent } else { LocalKind::LoopVar };
                if elem_kind == LocalKind::LentMut {
                    ctx.locals[id.0 as usize].kind = LocalKind::LentMut;
                }
                ctx.loop_depth += 1;
                let b = self.block(ctx, body)?;
                ctx.loop_depth -= 1;
                if !perm {
                    ctx.pop_scope();
                }
                Ok(hir::StmtKind::ForEach { elem: id, pos: pos_id, iter: it, body: b, perm, seq, para, nocheat: false })
            }
        }
    }

    /// A `const` in a body (owner, 2026-10-10: "I want it.", "`a`"): as a
    /// top-level one — its value known at compile time, nobody owning it —
    /// seen only in its block, never assigned and never shadowed.
    fn local_const(&mut self, ctx: &mut FnCtx, d: &ast::ConstDecl, span: Span) -> Result<hir::StmtKind> {
        let name = &d.name;
        if ctx.lookup_const(&name.name).is_some() {
            return err(Diagnostic::error(format!("'{}' is a const and cannot be shadowed: it is the same value everywhere it is seen", name.name)).named("const_shadowed").at(name.span));
        }
        if let Some(prev) = ctx.lookup(&name.name) {
            return err(Diagnostic::error(format!("'{}' is already declared; a const is a new name, and shadowing is never silent", name.name)).named("already_declared").at(name.span).why_at(ctx.locals[prev.0 as usize].span, "declared here"));
        }
        // Its value sees the consts around it and none of the variables.
        let mut cctx = self.new_ctx(crate::hir::FnInst(u32::MAX), ctx.module, "the const".to_string(), Ty::Nothing, Vec::new(), false);
        cctx.fold_vars = ctx.fold_vars;
        let mut seen = std::collections::HashMap::new();
        for s in &ctx.const_scopes {
            seen.extend(s.iter().map(|(k, v)| (k.clone(), *v)));
        }
        cctx.const_scopes = vec![seen];
        cctx.outer_vars = Some(ctx.scopes.iter().flat_map(|s| s.keys().cloned()).collect());
        cctx.in_const = true;
        let mut value = self.expr(&mut cctx, &d.value)?;
        let ty = match &d.ty {
            Some(t) => {
                let t = self.body_type(ctx, t)?;
                value = self.expect_store(&mut cctx, value, &t, "the const's declared type")?;
                t
            }
            None => self.resolve(&value.ty),
        };
        self.check_const_value(&value)?;
        let id = crate::hir::ConstId(self.items.consts.len() as u32);
        self.items.consts.push(crate::resolve::ConstInfo { name: name.name.clone(), module: ctx.module, vis: ast::Vis::Private, ty: Some(ty), value: d.value.clone(), span: name.span });
        self.const_values.push(Some(value));
        let key = if ctx.fold_vars { ast::fold_case(&name.name) } else { name.name.clone() };
        ctx.const_scopes.last_mut().unwrap().insert(key, id);
        Ok(hir::StmtKind::Block(hir::Block { stmts: Vec::new(), span }))
    }

    /// A `for` over an iterator (owner, 2026-10-08/09): a value whose type
    /// has its own `next`, taking it `lent.mut` and returning
    /// `optional[…]`, taken by `lend.mut` (it remains, used up as far as
    /// the loop went) or `give` (gone after). `next` runs once per trip, so
    /// `break` ends even an endless one.
    #[allow(clippy::too_many_arguments)]
    fn for_next(&mut self, ctx: &mut FnCtx, perm: bool, name: &ast::QName, pos: Option<&ast::QName>, e: &ast::Expr, it: hir::Expr, it_ty: &Ty, body: &ast::Block) -> Result<hir::StmtKind> {
        let ty_name = self.spell(it_ty);
        let key = match it_ty.base() {
            Ty::Struct { id, .. } => crate::resolve::ItemRef::Struct(*id),
            Ty::Enum { id, .. } => crate::resolve::ItemRef::Enum(*id),
            _ => unreachable!(),
        };
        let head = ty_name.split('[').next().unwrap_or(&ty_name).to_string();
        let Some(&next) = self.items.type_fns.get(&(key, "next".to_string())) else {
            return err(Diagnostic::error(format!("a `for` over {}, which has no `fn {}.next`: `for` walks a list, or an iterator", crate::types::a_type(&ty_name), head)).named("not_walkable").at(e.span));
        };
        let decl = self.items.fns[next.0 as usize].clone();
        let optional = self.items.base_optional;
        let shaped = decl.params.len() == 1 && decl.params[0].kind == crate::types::ParamKind::LentMut && matches!(&decl.ret, Ty::Enum { id, .. } if Some(*id) == optional);
        if !shaped {
            return err(Diagnostic::error(format!("`fn {}.next` does not walk: an iterator's `next` takes it `lent.mut` and returns `optional[…]`", head)).named("iterator_next_form").at(e.span).why_at(decl.name_span, format!("`fn {}.next` is declared here", head)));
        }
        if pos.is_some() {
            return err(Diagnostic::error("an iterator has no positions: its items come one at a time, so the loop takes no `pos`").named("iterator_has_no_position").at(e.span));
        }
        let mut stmts = Vec::new();
        // What `next` is called on: the lent place, or the iterator handed
        // over to the loop, held where nothing else can name it.
        let arg = match &it.kind {
            hir::ExprKind::Lend { mutable: true, .. } => e.clone(),
            hir::ExprKind::Lend { .. } => {
                return err(Diagnostic::error(format!("`next` changes the iterator, so a `for` takes {} with `lend.mut` (it remains) or `give` (it is gone after), not `lend`", crate::types::a_type(&ty_name))).named("lend_mut_needed").at(e.span));
            }
            _ => {
                let held = format!(" iterator {}", ctx.locals.len());
                let local = ctx.declare(&held, Local { name: held.clone(), ty: it_ty.clone(), mutability: Some(crate::ast::Mutability::Mut), kind: LocalKind::Owned, from_blank: false, span: e.span });
                stmts.push(hir::Stmt { kind: hir::StmtKind::Let { local, value: it, typed: true }, span: e.span, tier: ctx.tier });
                let q = ast::QName { name: held, span: e.span };
                ast::Expr { kind: ast::ExprKind::Verb { verb: ast::Verb::LendMut, amp: false, expr: Box::new(ast::Expr { kind: ast::ExprKind::Name(q), span: e.span }) }, span: e.span }
            }
        };
        let call = self.fn_call(ctx, next, &[ast::Arg::Positional(arg)], None, e.span, true, &ast::Path { kit: None, segs: Vec::new(), span: e.span })?;
        let call_ty = self.resolve(&call.ty);
        let Ty::Enum { id: oid, args } = call_ty.base() else { unreachable!() };
        let item_ty = args.first().cloned().unwrap_or(Ty::Nothing);
        let cases = &self.items.enums[oid.0 as usize].cases;
        let value_case = cases.iter().position(|c| c.name == "value").unwrap();
        let none_case = cases.iter().position(|c| c.name == "none").unwrap();
        if !perm {
            ctx.push_scope();
        }
        let elem = ctx.declare(&name.name, Local { name: name.name.clone(), ty: item_ty, mutability: None, kind: LocalKind::LoopVar, from_blank: false, span: name.span });
        ctx.loop_depth += 1;
        let b = self.block(ctx, body)?;
        ctx.loop_depth -= 1;
        if !perm {
            ctx.pop_scope();
        }
        stmts.push(hir::Stmt { kind: hir::StmtKind::ForNext { elem, call, value_case, none_case, body: b, perm }, span: body.span, tier: ctx.tier });
        Ok(hir::StmtKind::Block(hir::Block { stmts, span: body.span }))
    }

    /// The element type of something a `for` can run over.
    pub fn elem_type_of(&self, t: &Ty, span: Span, what: &str) -> Result<Ty> {
        match t.base() {
            Ty::Seq { elem, .. } => Ok((**elem).clone()),
            Ty::Bundle(ts) => Ok(Ty::Bundle(ts.clone())),
            Ty::Str(_) => err(Diagnostic::error("a text is not a list of anything until you say what: `str.each.grapheme(lend $s)`, `.point`, or `.byte`").named("text_unit_unsaid").at(span)),
            other => err(Diagnostic::error(format!("{} {} is not possible: a `for` needs a list", what, crate::types::a_type(&self.spell(other)))).named("not_walkable").at(span)),
        }
    }

    fn match_stmt(&mut self, ctx: &mut FnCtx, value: &ast::Expr, arms: &[ast::MatchArm]) -> Result<hir::StmtKind> {
        // A case that gives a value needs a `match` used as one.
        for arm in arms {
            if let Some(g @ ast::Stmt { kind: ast::StmtKind::Gives { .. }, .. }) = arm.body.stmts.last() {
                return err(Diagnostic::error(format!("the `{}` case gives a value, but this `match` is not used as one: `let $x := match … {{ … }};`", arm.case.name)).named("match_value_unused").at(g.span));
            }
        }
        let (v, out) = self.match_arms(ctx, value, arms, None)?;
        Ok(hir::StmtKind::Match { value: v, arms: out })
    }

    /// A `match` used as a value (owner, 2026-09-29: a): each case gives
    /// it, or leaves (`return`, `fail`, `break`, `continue`).
    pub fn match_value(&mut self, ctx: &mut FnCtx, value: &ast::Expr, arms: &[ast::MatchArm], span: crate::source::Span) -> Result<hir::Expr> {
        let t = self.fresh();
        let (v, out) = self.match_arms(ctx, value, arms, Some(&t))?;
        let ty = self.resolve(&t);
        Ok(hir::Expr { kind: hir::ExprKind::Match { value: Box::new(v), arms: out }, ty, span })
    }

    fn match_arms(&mut self, ctx: &mut FnCtx, value: &ast::Expr, arms: &[ast::MatchArm], gives: Option<&Ty>) -> Result<(hir::Expr, Vec<hir::MatchArm>)> {
        let v = self.expr(ctx, value)?;
        let vt = self.resolve(&v.ty);
        let (id, args) = match vt.base() {
            Ty::Enum { id, args } => (*id, args.clone()),
            other => return err(Diagnostic::error(format!("`match` takes an enum value, and this is {}: use `if`", crate::types::a_type(&self.spell(other)))).named("match_needs_enum").at(value.span)),
        };
        let info = self.items.enums[id.0 as usize].clone();
        let mut out = Vec::new();
        let mut covered = vec![false; info.cases.len()];
        for arm in arms {
            let Some(ci) = info.cases.iter().position(|c| self.items.same_name(&self.items.case_cases, info.span, &c.name, &arm.case.name)) else {
                return err(Diagnostic::error(format!("`{}` has no case `{}`: its cases are {}", info.name, arm.case.name, info.cases.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", "))).named("unknown_case").at(arm.case.span).fix_close(arm.case.span, &arm.case.name, info.cases.iter().map(|c| c.name.as_str())));
            };
            if covered[ci] {
                return err(Diagnostic::error(format!("the case `{}` is handled twice", arm.case.name)).named("case_handled_twice").at(arm.case.span));
            }
            covered[ci] = true;
            ctx.push_scope();
            let bind = match (&arm.bind, &info.cases[ci].payload) {
                (None, None) => None,
                (Some(b), Some(t)) => {
                    let t = t.subst(&args);
                    // The bound name looks at the payload: lent from the matched value.
                    let kind = match &v.kind {
                        hir::ExprKind::Give(_) | hir::ExprKind::Copy(_) => LocalKind::Owned,
                        _ => LocalKind::Lent,
                    };
                    Some(ctx.declare(&b.name, Local { name: b.name.clone(), ty: t, mutability: None, kind, from_blank: false, span: b.span }))
                }
                (Some(b), None) => return err(Diagnostic::error(format!("`{}` carries nothing, so there is no value to name '{}'", arm.case.name, b.name)).named("carries_nothing").at(b.span)),
                (None, Some(t)) => return err(Diagnostic::error(format!("`{}` carries {}: name it — `{} $value {{ … }}`", arm.case.name, crate::types::a_type(&self.spell(t)), arm.case.name)).named("data_missing").at(arm.case.span)),
            };
            let (body, given) = match gives {
                Some(t) => {
                    let (body, g) = self.arm_block(ctx, &arm.body)?;
                    let g = match g {
                        Some(g) => Some(self.expect_store(ctx, g, t, "the `match`'s value")?),
                        None if super::stmt::leaves_block(&body) => None,
                        None => return err(Diagnostic::error(format!("the `{}` case gives no value: end it with one (`{{ … }}`), or leave with `return`, `fail` or `break`", arm.case.name)).named("case_value_missing").at(arm.body.span)),
                    };
                    (body, g)
                }
                None => (self.block(ctx, &arm.body)?, None),
            };
            ctx.pop_scope();
            out.push(hir::MatchArm { case: ci, bind, body, gives: given });
        }
        let missing: Vec<&str> = info.cases.iter().enumerate().filter(|(i, _)| !covered[*i]).map(|(_, c)| c.name.as_str()).collect();
        if !missing.is_empty() {
            return err(Diagnostic::error(format!("this `match` does not handle {}: every case of `{}` must have an arm", missing.iter().map(|m| format!("`{}`", m)).collect::<Vec<_>>().join(", "), info.name)).named("case_unhandled").at(value.span));
        }
        Ok((v, out))
    }

    fn concur_stmt(&mut self, ctx: &mut FnCtx, detached: bool, captures: &[ast::Capture], body: &ast::ConcurBody, otherwise: Option<&ast::Otherwise>, span: Span) -> Result<hir::StmtKind> {
        // Captures are resolved outside; the task sees only them.
        let mut inner_locals: Vec<(String, ast::Verb, Ty, LocalId)> = Vec::new();
        for c in captures {
            let Some(outer) = ctx.lookup(&c.name.name) else {
                return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared", c.name.name)).named("not_declared").at(c.name.span), &c.name.name, c.name.span));
            };
            if detached && matches!(c.verb, ast::Verb::Lend | ast::Verb::LendMut) {
                return err(Diagnostic::error(format!("a detached task may not lend: no scope protects it, so '{}' could be gone while the task still looks at it — `give` or `copy` it", c.name.name)).named("mcps::detached_lend").at(c.span));
            }
            let ty = ctx.locals[outer.0 as usize].ty.clone();
            if c.verb == ast::Verb::LendMut {
                ctx.changed.insert(outer);
            }
            inner_locals.push((c.name.name.clone(), c.verb, ty, outer));
        }
        let body_block = match body {
            ast::ConcurBody::Block(b) => b.clone(),
            ast::ConcurBody::Call(e) => ast::Block { stmts: vec![ast::Stmt { kind: ast::StmtKind::Expr(e.clone()), span: e.span }], span: e.span },
        };
        // The task's body is checked in a fresh scope holding only the captures;
        // whatever it passes up with `otherwise fail` collects in `ctx.fails`.
        let saved_scopes = std::mem::replace(&mut ctx.scopes, vec![std::collections::HashMap::new()]);
        let saved_task = ctx.is_task;
        let saved_loop = ctx.loop_depth;
        let saved_fails = std::mem::take(&mut ctx.fails);
        let saved_ret = std::mem::replace(&mut ctx.ret, Ty::Nothing);
        ctx.is_task = true;
        ctx.loop_depth = 0;
        let mut caps = Vec::new();
        for (name, verb, ty, outer) in &inner_locals {
            let kind = match verb {
                ast::Verb::Lend => LocalKind::Lent,
                ast::Verb::LendMut => LocalKind::LentMut,
                _ => LocalKind::Owned,
            };
            let inner = ctx.declare(name, Local { name: name.clone(), ty: ty.clone(), mutability: None, kind, from_blank: false, span });
            caps.push(hir::CaptureBinding { verb: *verb, outer: *outer, inner });
        }
        let b = self.block(ctx, &body_block)?;
        let fails = std::mem::take(&mut ctx.fails);
        ctx.scopes = saved_scopes;
        ctx.is_task = saved_task;
        ctx.loop_depth = saved_loop;
        ctx.fails = saved_fails;
        ctx.ret = saved_ret;
        let ow = match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error("this task cannot fail; drop the `otherwise`").named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                Some(self.otherwise(ctx, o, &fails, &Ty::Nothing, false, span)?)
            }
            None => {
                if !fails.is_empty() {
                    return err(Diagnostic::error(format!("this `concur` can fail ({}), so it needs an `otherwise` saying what happens when it does", fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", "))).named("failure_unhandled").at(span));
                }
                None
            }
        };
        Ok(hir::StmtKind::Concur { detached, captures: caps, body: b, otherwise: ow, fails })
    }

    fn whichever_stmt(&mut self, ctx: &mut FnCtx, arms: &[ast::WhicheverArm], otherwise: Option<&ast::Otherwise>, span: Span) -> Result<hir::StmtKind> {
        let mut out = Vec::new();
        for arm in arms {
            let ch = self.expr(ctx, &arm.channel)?;
            let ct = self.resolve(&ch.ty);
            let elem = match ct.base() {
                Ty::Channel(e) => (**e).clone(),
                other => return err(Diagnostic::error(format!("an arm of `whichever` takes from a channel, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(arm.channel.span)),
            };
            if !matches!(ch.kind, hir::ExprKind::Lend { .. }) {
                return err(Diagnostic::error("a channel is lent to `whichever`: `$v from lend $c { … }`").named("verb_wrong_for_parameter").at(arm.channel.span));
            }
            ctx.push_scope();
            let id = ctx.declare(&arm.name.name, Local { name: arm.name.name.clone(), ty: elem, mutability: None, kind: LocalKind::Owned, from_blank: false, span: arm.name.span });
            let body = self.block(ctx, &arm.body)?;
            ctx.pop_scope();
            out.push(hir::WhicheverArm { value: id, channel: ch, body });
        }
        let fails = vec![("closed".to_string(), None)];
        let ow = match otherwise {
            Some(o) => Some(self.otherwise(ctx, o, &fails, &Ty::Nothing, false, span)?),
            None => return err(Diagnostic::error("`whichever` can fail (closed) when every channel is closed, so it needs an `otherwise`").named("failure_unhandled").at(span)),
        };
        Ok(hir::StmtKind::Whichever { arms: out, otherwise: ow })
    }
}

pub use crate::source::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceUse {
    Read,
    Write,
    Lend,
    LendMut,
    Give,
    Copy,
}
