//! Statements.

use crate::ast::{self, Mutability, Tier};
use crate::diag::{err, Diagnostic, Result};
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
        let tier = ctx.tier;
        let kind = self.stmt_kind(ctx, s)?;
        Ok(hir::Stmt { kind, span: s.span, tier })
    }

    fn stmt_kind(&mut self, ctx: &mut FnCtx, s: &ast::Stmt) -> Result<hir::StmtKind> {
        match &s.kind {
            ast::StmtKind::Tier { tier, stmt } => {
                let saved = ctx.tier;
                ctx.tier = Some(*tier);
                let inner = self.stmt(ctx, stmt)?;
                ctx.tier = saved;
                let _ = Tier::Check;
                Ok(inner.kind)
            }
            ast::StmtKind::Let { mutability, name, ty, shadow, value } => {
                // Shadowing is never silent.
                if let Some(prev) = ctx.in_current_scope(&name.name) {
                    let p = &ctx.locals[prev.0 as usize];
                    if *shadow {
                        if p.mutability == Some(Mutability::Immut) {
                            return err(Diagnostic::error(format!("'{}' is `immut` and cannot be shadowed", name.name)).at(name.span).why_at(p.span, "declared `immut` here"));
                        }
                    } else {
                        return err(Diagnostic::error(format!("'{}' is already declared in this scope; to shadow it deliberately write `$:=` or `$=`", name.name)).at(name.span).why_at(p.span, "the first '".to_string() + &name.name + "' is declared here"));
                    }
                } else if let Some(prev) = ctx.visible_outside(&name.name) {
                    let p = &ctx.locals[prev.0 as usize];
                    if *shadow {
                        if p.mutability == Some(Mutability::Immut) {
                            return err(Diagnostic::error(format!("'{}' is `immut` and cannot be shadowed", name.name)).at(name.span).why_at(p.span, "declared `immut` here"));
                        }
                    } else {
                        return err(Diagnostic::error(format!("'{}' is already declared in an enclosing scope; to shadow it deliberately write `$:=` or `$=`", name.name)).at(name.span).why_at(p.span, "the first '".to_string() + &name.name + "' is declared here"));
                    }
                } else if *shadow {
                    if self.const_visible(ctx, &name.name) {
                        return err(Diagnostic::error(format!("'{}' is a const and cannot be shadowed", name.name)).at(name.span));
                    }
                    return err(Diagnostic::error(format!("`$` shadows a visible '{}', and none is visible here: write `:=`", name.name)).at(name.span));
                }
                if !*shadow && self.const_visible(ctx, &name.name) {
                    return err(Diagnostic::error(format!("'{}' is a const of this Limb and cannot be redeclared", name.name)).at(name.span));
                }
                let declared = match ty {
                    Some(t) => {
                        let mut blanks = Vec::new();
                        let rt = self.items.resolve_type(t, ctx.module, &mut blanks, false)?;
                        Some(rt)
                    }
                    None => None,
                };
                if let Some(t) = &declared {
                    if let Ty::Ptr { .. } = t.base() {
                        return err(Diagnostic::error(format!("`{}` exists only at the boundary: in an `extern fn` signature or a `layout.c` field — a `let` cannot declare a pointer; a pointer comes only from a lend or from C", self.spell(t))).at(ty.as_ref().unwrap().span));
                    }
                }
                let v = self.expr_top(ctx, value, true)?;
                if let hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::ListAppend), .. } = &v.kind {
                    return err(Diagnostic::error("`list.append` hands nothing back and cannot be used as a value: it changes the list in place").at(value.span));
                }
                if let (hir::ExprKind::List(items), None) = (&v.kind, &declared) {
                    if items.is_empty() {
                        return err(Diagnostic::error(format!("`[]` says nothing about what '{}' holds, and nothing on the left says \"list\": write `list.empty` (Precog picks the element type from what is appended later), or give it a type", name.name)).at(value.span));
                    }
                }
                let (v, local_ty, kind) = match &declared {
                    Some(t) => {
                        if let (hir::ExprKind::List(items), Ty::Seq { kind: SeqKind::Array, .. }) = (&v.kind, self.resolve(t).base()) {
                            if items.is_empty() {
                                return err(Diagnostic::error(format!("an empty `array` could never be filled: an array is fixed at the size it was made with — make '{}' a `list`, or give it its elements", name.name)).at(value.span));
                            }
                        }
                        let v = self.expect_store(ctx, v, t, "the declared type")?;
                        self.check_stored_value(ctx, &v, t)?;
                        (v, t.clone(), LocalKind::Owned)
                    }
                    None => {
                        let vt = self.resolve(&v.ty);
                        if vt == Ty::Nothing {
                            return err(Diagnostic::error(format!("'{}' would hold nothing: the value on the right returns nothing", name.name)).at(value.span));
                        }
                        let kind = match &v.kind {
                            hir::ExprKind::Lend { mutable, .. } => if *mutable { LocalKind::LentMut } else { LocalKind::Lent },
                            _ => LocalKind::Owned,
                        };
                        self.check_stored_value(ctx, &v, &vt)?;
                        (v, self.infer_local_type(&vt), kind)
                    }
                };
                let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: local_ty, mutability: *mutability, kind, from_blank: false, span: name.span });
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
                let v = self.expr_top(ctx, value, true)?;
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
            ast::StmtKind::For { seq, perm, name, ty, iter, body } => self.for_stmt(ctx, *seq, *perm, name, ty.as_ref(), iter, body),
            ast::StmtKind::Return(value) => {
                if ctx.is_task {
                    return err(Diagnostic::error("a task is a block, not a function: `return` cannot leave it").at(s.span));
                }
                if ctx.is_main {
                    return err(Diagnostic::error("`MAIN` is a block, not a function: it has no `return` — ending it exits 0, and any other code is `std::exit[n]`").at(s.span));
                }
                match value {
                    Some(v) => {
                        if ctx.ret == Ty::Nothing {
                            return err(Diagnostic::error(format!("`{}` returns nothing, so `return` takes no value: write `return;` or give the function a `-> type`", ctx.name)).at(v.span));
                        }
                        let e = self.expr_top(ctx, v, true)?;
                        let rt = ctx.ret.clone();
                        let e = self.expect_store(ctx, e, &rt, "the return type")?;
                        self.check_stored_value(ctx, &e, &rt)?;
                        Ok(hir::StmtKind::Return(Some(e)))
                    }
                    None => {
                        if ctx.ret != Ty::Nothing {
                            return err(Diagnostic::error(format!("`{}` returns a `{}`: `return` needs the value", ctx.name, self.spell(&ctx.ret))).at(s.span));
                        }
                        Ok(hir::StmtKind::Return(None))
                    }
                }
            }
            ast::StmtKind::Fail { name, payload } => self.fail_stmt(ctx, name, payload.as_ref(), s.span),
            ast::StmtKind::Break => {
                if ctx.loop_depth == 0 {
                    return err(Diagnostic::error("`break` outside a loop: there is nothing to leave").at(s.span));
                }
                Ok(hir::StmtKind::Break)
            }
            ast::StmtKind::Continue => {
                if ctx.loop_depth == 0 {
                    return err(Diagnostic::error("`continue` outside a loop: there is nothing to continue").at(s.span));
                }
                Ok(hir::StmtKind::Continue)
            }
            ast::StmtKind::Block(b) => Ok(hir::StmtKind::Block(self.block(ctx, b)?)),
            ast::StmtKind::Match { value, arms } => self.match_stmt(ctx, value, arms),
            ast::StmtKind::Concur { detached, captures, body, otherwise } => self.concur_stmt(ctx, *detached, captures, body, otherwise.as_ref(), s.span),
            ast::StmtKind::Whichever { arms, otherwise } => self.whichever_stmt(ctx, arms, otherwise.as_ref(), s.span),
            ast::StmtKind::InlineAsm { text, bindings } => {
                if ctx.tier != Some(Tier::Nocheck) {
                    return err(Diagnostic::error("an inline block is a nocheck site: nothing inside it can be proven — write `nocheck inline.ASM { … }`").at(s.span));
                }
                let mut ins = Vec::new();
                let mut outs = Vec::new();
                let mut clobbers = Vec::new();
                for b in bindings {
                    match b {
                        ast::AsmBinding::Clobber(r) => clobbers.push(r.name.clone()),
                        ast::AsmBinding::Reg { name, dir, reg } => {
                            match dir {
                                ast::AsmDir::In => {
                                    let Some(id) = ctx.lookup(&name.name) else {
                                        return err(Diagnostic::error(format!("'{}' is not declared", name.name)).at(name.span));
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
                                        return err(Diagnostic::error(format!("'{}' is not declared", name.name)).at(name.span));
                                    };
                                    ins.push((id, reg.name.clone()));
                                    outs.push((id, reg.name.clone()));
                                }
                            }
                        }
                    }
                }
                Ok(hir::StmtKind::InlineAsm { text: text.clone(), ins, outs, clobbers })
            }
        }
    }

    fn const_visible(&self, ctx: &FnCtx, name: &str) -> bool {
        self.items.modules[ctx.module.0 as usize].consts.contains_key(name)
    }

    /// The type a `:=` local takes from its value: a fixed source keeps its
    /// type; a free value stays free for Precog to size.
    fn infer_local_type(&self, vt: &Ty) -> Ty {
        vt.clone()
    }

    pub fn expect_bool(&self, e: &hir::Expr, what: &str) -> Result<()> {
        let t = self.resolve(&e.ty);
        if t.base() != &Ty::Bool {
            return err(Diagnostic::error(format!("{} must be a `bool`, and this is a `{}`: there is no truthiness — compare it (`'n' !== 0`)", what, self.spell(&t))).at(e.span));
        }
        Ok(())
    }

    fn fail_stmt(&mut self, ctx: &mut FnCtx, name: &ast::Ident, payload: Option<&ast::Expr>, span: Span) -> Result<hir::StmtKind> {
        if ctx.is_main {
            return err(Diagnostic::error("`MAIN` cannot fail: it exits — `std::exit[1]` says the code").at(span));
        }
        let Some(index) = ctx.fails.iter().position(|(n, _)| *n == name.name) else {
            if ctx.fails.is_empty() {
                return err(Diagnostic::error(format!("`{}` cannot fail: its signature has no `fail[…]` — add `-> … or fail[{}]` to say it can", ctx.name, name.name)).at(name.span));
            }
            return err(Diagnostic::error(format!("`{}` does not list `{}` among its failures: its signature says fail[{}]", ctx.name, name.name, ctx.fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", "))).at(name.span));
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
            (None, Some(t)) => return err(Diagnostic::error(format!("`{}` carries a `{}`, and `fail {}` needs it: `fail {}[…]`", name.name, self.spell(&t), name.name, name.name)).at(span)),
            (Some(p), None) => return err(Diagnostic::error(format!("`{}` carries nothing: write `fail {};` with no value", name.name, name.name)).at(p.span)),
        };
        Ok(hir::StmtKind::Fail { name: name.name.clone(), index, payload })
    }

    fn for_stmt(&mut self, ctx: &mut FnCtx, seq: bool, perm: bool, name: &ast::QName, ty: Option<&ast::Type>, iter: &ast::ForIter, body: &ast::Block) -> Result<hir::StmtKind> {
        if let Some(prev) = ctx.lookup(&name.name) {
            let p = &ctx.locals[prev.0 as usize];
            return err(Diagnostic::error(format!("'{}' is already declared; a loop variable is a new name (shadowing is never silent)", name.name)).at(name.span).why_at(p.span, "declared here"));
        }
        match iter {
            ast::ForIter::Range { start, stop, step } => {
                let counter_ty = match ty {
                    Some(t) => {
                        let mut blanks = Vec::new();
                        let rt = self.items.resolve_type(t, ctx.module, &mut blanks, false)?;
                        if !rt.is_int() {
                            return err(Diagnostic::error(format!("a range counter is an integer; `{}` is not", self.spell(&rt))).at(t.span));
                        }
                        rt
                    }
                    None => Ty::Int(IntW::Free(InferId(0))),
                };
                let s = self.expr(ctx, start)?;
                let s = self.expect_store(ctx, s, &counter_ty, "the counter")?;
                let e = self.expr(ctx, stop)?;
                let e = self.expect_store(ctx, e, &counter_ty, "the counter")?;
                let st = match step {
                    Some(st) => {
                        let v = self.expr(ctx, st)?;
                        let v = self.expect_store(ctx, v, &counter_ty, "the step")?;
                        if let hir::ExprKind::Int(0) = v.kind {
                            return err(Diagnostic::error("a step of 0 would never end: the counter must move").at(st.span));
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
                Ok(hir::StmtKind::ForRange { counter: id, start: s, stop: e, step: st, body: b, perm, seq })
            }
            ast::ForIter::Over(e) => {
                if ty.is_some() {
                    return err(Diagnostic::error("a list loop's variable takes no type: each element's type is the list's").at(name.span));
                }
                let it = self.expr(ctx, e)?;
                let it_ty = self.resolve(&it.ty);
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
                        _ => return err(Diagnostic::error("a `for` runs over a list (`lend 'xs'` / `give 'xs'`), a `range[…]`, a text walk (`str.each.grapheme[…]`) or a map walk (`map.keys[…]`)").at(e.span)),
                    },
                    hir::ExprKind::Local(_) | hir::ExprKind::Read(_) if it_ty.is_seq() || matches!(it_ty.base(), Ty::Bundle(_)) => {
                        return err(Diagnostic::error(format!("say how the loop takes the list: `for '{}' in lend 'xs'` looks at each element, `give 'xs'` consumes the list", name.name)).at(e.span));
                    }
                    _ if it_ty.is_text() => {
                        return err(Diagnostic::error("a text is not a list of anything until you say what: `str.each.grapheme[lend 's']`, `.point`, or `.byte`").at(e.span));
                    }
                    _ => return err(Diagnostic::error(format!("a `for` needs a list to run over, and this is a `{}`", self.spell(&it_ty))).at(e.span)),
                };
                // A mixed bundle: the body is checked once per element, unrolled.
                if let Ty::Bundle(ts) = self.resolve(&elem_ty) {
                    if perm {
                        return err(Diagnostic::error("`perm` on a loop over a mixed bundle: the elements have different types, so there is no one variable to keep").at(name.span));
                    }
                    let place = match &it.kind {
                        hir::ExprKind::Lend { place, .. } => place.clone(),
                        _ => return err(Diagnostic::error("a mixed bundle is walked with `lend`").at(e.span)),
                    };
                    let mut stmts = Vec::new();
                    for (i, t) in ts.iter().enumerate() {
                        ctx.push_scope();
                        let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: t.clone(), mutability: None, kind: LocalKind::LoopVar, from_blank: false, span: name.span });
                        let value = hir::Expr { kind: hir::ExprKind::BundleElem { place: place.clone(), index: i }, ty: t.clone(), span: e.span };
                        let saved_depth = ctx.loop_depth;
                        ctx.loop_depth = 0; // break/continue are not offered inside an unrolled walk
                        let b = self.block(ctx, body)?;
                        ctx.loop_depth = saved_depth;
                        ctx.pop_scope();
                        let mut inner = vec![hir::Stmt { kind: hir::StmtKind::Let { local: id, value, typed: false }, span: name.span, tier: ctx.tier }];
                        inner.push(hir::Stmt { kind: hir::StmtKind::Block(b), span: body.span, tier: ctx.tier });
                        stmts.push(hir::Stmt { kind: hir::StmtKind::Block(hir::Block { stmts: inner, span: body.span }), span: body.span, tier: ctx.tier });
                    }
                    return Ok(hir::StmtKind::Block(hir::Block { stmts, span: body.span }));
                }
                if !perm {
                    ctx.push_scope();
                }
                let id = ctx.declare(&name.name, Local { name: name.name.clone(), ty: elem_ty, mutability: None, kind: if elem_kind == LocalKind::Owned { LocalKind::LoopVar } else { elem_kind }, from_blank: false, span: name.span });
                // A lent element is a loop variable too: not assignable.
                ctx.locals[id.0 as usize].kind = LocalKind::LoopVar;
                if elem_kind == LocalKind::LentMut {
                    ctx.locals[id.0 as usize].kind = LocalKind::LentMut;
                }
                ctx.loop_depth += 1;
                let b = self.block(ctx, body)?;
                ctx.loop_depth -= 1;
                if !perm {
                    ctx.pop_scope();
                }
                Ok(hir::StmtKind::ForEach { elem: id, iter: it, body: b, perm, seq })
            }
        }
    }

    /// The element type of something a `for` can run over.
    pub fn elem_type_of(&self, t: &Ty, span: Span, what: &str) -> Result<Ty> {
        match t.base() {
            Ty::Seq { elem, .. } => Ok((**elem).clone()),
            Ty::Bundle(ts) => Ok(Ty::Bundle(ts.clone())),
            Ty::Str(_) => err(Diagnostic::error("a text is not a list of anything until you say what: `str.each.grapheme[lend 's']`, `.point`, or `.byte`").at(span)),
            other => err(Diagnostic::error(format!("{} a `{}` is not possible: a `for` needs a list", what, self.spell(other))).at(span)),
        }
    }

    fn match_stmt(&mut self, ctx: &mut FnCtx, value: &ast::Expr, arms: &[ast::MatchArm]) -> Result<hir::StmtKind> {
        let v = self.expr(ctx, value)?;
        let vt = self.resolve(&v.ty);
        let (id, args) = match vt.base() {
            Ty::Enum { id, args } => (*id, args.clone()),
            other => return err(Diagnostic::error(format!("`match` takes an enum value, and this is a `{}`: use `if`", self.spell(other))).at(value.span)),
        };
        let info = self.items.enums[id.0 as usize].clone();
        let mut out = Vec::new();
        let mut covered = vec![false; info.cases.len()];
        for arm in arms {
            let Some(ci) = info.cases.iter().position(|c| c.name == arm.case.name) else {
                return err(Diagnostic::error(format!("`{}` has no case `{}`: its cases are {}", info.name, arm.case.name, info.cases.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", "))).at(arm.case.span));
            };
            if covered[ci] {
                return err(Diagnostic::error(format!("the case `{}` is handled twice", arm.case.name)).at(arm.case.span));
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
                (Some(b), None) => return err(Diagnostic::error(format!("`{}` carries nothing, so there is no value to name '{}'", arm.case.name, b.name)).at(b.span)),
                (None, Some(t)) => return err(Diagnostic::error(format!("`{}` carries a `{}`: name it — `{} 'value' {{ … }}`", arm.case.name, self.spell(t), arm.case.name)).at(arm.case.span)),
            };
            let body = self.block(ctx, &arm.body)?;
            ctx.pop_scope();
            out.push(hir::MatchArm { case: ci, bind, body });
        }
        let missing: Vec<&str> = info.cases.iter().enumerate().filter(|(i, _)| !covered[*i]).map(|(_, c)| c.name.as_str()).collect();
        if !missing.is_empty() {
            return err(Diagnostic::error(format!("this `match` does not handle {}: every case of `{}` must have an arm", missing.iter().map(|m| format!("`{}`", m)).collect::<Vec<_>>().join(", "), info.name)).at(value.span));
        }
        Ok(hir::StmtKind::Match { value: v, arms: out })
    }

    fn concur_stmt(&mut self, ctx: &mut FnCtx, detached: bool, captures: &[ast::Capture], body: &ast::ConcurBody, otherwise: Option<&ast::Otherwise>, span: Span) -> Result<hir::StmtKind> {
        // Captures are resolved outside; the task sees only them.
        let mut inner_locals: Vec<(String, ast::Verb, Ty, LocalId)> = Vec::new();
        for c in captures {
            let Some(outer) = ctx.lookup(&c.name.name) else {
                return err(Diagnostic::error(format!("'{}' is not declared", c.name.name)).at(c.name.span));
            };
            if c.amp {
                self.amp_count += 1;
            }
            if detached && matches!(c.verb, ast::Verb::Lend | ast::Verb::LendMut) {
                return err(Diagnostic::error(format!("a detached task may not lend: no scope protects it, so '{}' could be gone while the task still looks at it — `give` or `copy` it", c.name.name)).at(c.span));
            }
            let ty = ctx.locals[outer.0 as usize].ty.clone();
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
                    return err(Diagnostic::error("this task cannot fail; drop the `otherwise`").at(span));
                }
                Some(self.otherwise(ctx, o, &fails, &Ty::Nothing, false, span)?)
            }
            None => {
                if !fails.is_empty() {
                    return err(Diagnostic::error(format!("this `concur` can fail ({}), so it needs an `otherwise` saying what happens when it does", fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", "))).at(span));
                }
                None
            }
        };
        Ok(hir::StmtKind::Concur { detached, captures: caps, body: b, otherwise: ow })
    }

    fn whichever_stmt(&mut self, ctx: &mut FnCtx, arms: &[ast::WhicheverArm], otherwise: Option<&ast::Otherwise>, span: Span) -> Result<hir::StmtKind> {
        let mut out = Vec::new();
        for arm in arms {
            let ch = self.expr(ctx, &arm.channel)?;
            let ct = self.resolve(&ch.ty);
            let elem = match ct.base() {
                Ty::Channel(e) => (**e).clone(),
                other => return err(Diagnostic::error(format!("an arm of `whichever` takes from a channel, and this is a `{}`", self.spell(other))).at(arm.channel.span)),
            };
            if !matches!(ch.kind, hir::ExprKind::Lend { .. }) {
                return err(Diagnostic::error("a channel is lent to `whichever`: `'v' from lend 'c' { … }`").at(arm.channel.span));
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
            None => return err(Diagnostic::error("`whichever` can fail (closed) when every channel is closed, so it needs an `otherwise`").at(span)),
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
