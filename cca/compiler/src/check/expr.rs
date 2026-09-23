//! Expressions, places, calls, verbs, conversions, `otherwise`.

use crate::ast::{self, Arg as AArg, BinOp, ExprKind as AK, UnOp, Verb};
use crate::diag::{err, Diagnostic, Result};
use crate::hir::{self, Arg, Builtin, Callee, Expr, ExprKind as K, Local, LocalKind, Place, Proj};
use crate::resolve::{FnId, ItemRef};
use crate::source::Span;
use crate::types::*;

use super::stmt::PlaceUse;
use super::{Checker, FnCtx, InstKey};

const FREE_INT: Ty = Ty::Int(IntW::Free(InferId(0)));
const FREE_BIN: Ty = Ty::Bin(BinW::Free(InferId(0)));

impl Checker {
    // ------------------------------------------------------------ entry points

    /// A value in a position where a failing call may carry `otherwise`.
    pub fn expr_top(&mut self, ctx: &mut FnCtx, e: &ast::Expr, used: bool) -> Result<Expr> {
        self.expr_in(ctx, e, true, used)
    }

    /// A value inside a larger expression: a failing call needs its own `let`.
    pub fn expr(&mut self, ctx: &mut FnCtx, e: &ast::Expr) -> Result<Expr> {
        self.expr_in(ctx, e, false, true)
    }

    /// A call in statement position.
    pub fn expr_stmt_value(&mut self, ctx: &mut FnCtx, e: &ast::Expr) -> Result<Expr> {
        let v = self.expr_in(ctx, e, true, false)?;
        match &v.kind {
            K::Call { .. } | K::Conv { .. } => {}
            _ => return err(Diagnostic::error("a value alone is not a statement: assign it, or call something with it").at(e.span)),
        }
        Ok(v)
    }

    fn expr_in(&mut self, ctx: &mut FnCtx, e: &ast::Expr, top: bool, used: bool) -> Result<Expr> {
        let span = e.span;
        match &e.kind {
            AK::Int(t) => {
                let v: i128 = t.parse().map_err(|_| Box::new(Diagnostic::error(format!("`{}` is too large for any integer type", t)).at(span)))?;
                Ok(Expr { kind: K::Int(v), ty: FREE_INT, span })
            }
            AK::Bin(t) => Ok(Expr { kind: K::Bin(t.clone()), ty: FREE_BIN, span }),
            AK::Str(s) => Ok(Expr { kind: K::Str(s.clone()), ty: Ty::Str(Enc::Free(InferId(0))), span }),
            AK::Newline => Ok(Expr { kind: K::Str("\n".into()), ty: Ty::Str(Enc::Free(InferId(0))), span }),
            AK::Char(c) => {
                let unit = narrowest_unit(c);
                let _ = unit;
                Ok(Expr { kind: K::Char(c.clone()), ty: Ty::Char(Unit::Free(InferId(0))), span })
            }
            AK::Bool(b) => Ok(Expr { kind: K::Bool(*b), ty: Ty::Bool, span }),
            AK::Duration { value, unit } => {
                let v: i128 = value.parse().map_err(|_| Box::new(Diagnostic::error("this duration is too large").at(span)))?;
                let nanos = match unit.name.trim_end_matches('s') {
                    "nano" => 1,
                    "micro" => 1_000,
                    "milli" => 1_000_000,
                    "second" => 1_000_000_000,
                    "minute" => 60_000_000_000,
                    "hour" => 3_600_000_000_000i128,
                    _ => unreachable!(),
                };
                Ok(Expr { kind: K::Duration(v * nanos), ty: Ty::Duration, span })
            }
            AK::Name(n) => {
                let Some(id) = ctx.lookup(&n.name) else {
                    // A const of this module, or C's variable.
                    if let Some((cid, _, _)) = self.items.modules[ctx.module.0 as usize].consts.get(&n.name).cloned() {
                        let ty = self.const_type(cid, span)?;
                        return Ok(Expr { kind: K::Const(cid), ty, span });
                    }
                    if let Some(x) = self.extern_let_named(&n.name, ctx) {
                        let place = self.extern_place(ctx, x, PlaceUse::Read, span)?;
                        let ty = place.ty.clone();
                        return Ok(Expr { kind: K::ExternRead(x), ty, span }).map(|e| { let _ = place; e });
                    }
                    return self.undeclared(ctx, n);
                };
                let local = &ctx.locals[id.0 as usize];
                let ty = local.ty.clone();
                let rt = self.resolve(&ty);
                if rt.is_owned() && !local.from_blank {
                    // A bare owned value in a value position: the caller decides
                    // (a verb is required to store or hand it over; reading a plain
                    // field through it is fine). Return a Read of the place; the
                    // consumer checks `check_stored_value`.
                    let place = Place { base: id, proj: Vec::new(), ty: ty.clone(), span };
                    return Ok(Expr { kind: K::Read(place), ty, span });
                }
                Ok(Expr { kind: K::Local(id), ty, span })
            }
            AK::ConstPath { path, name } => {
                let id = self.items.lookup_const_path(path, name, ctx.module)?;
                let ty = self.const_type(id, span)?;
                Ok(Expr { kind: K::Const(id), ty, span })
            }
            AK::Path(path) => self.path_value(ctx, path, span),
            AK::Call { path, args, otherwise } => {
                if otherwise.is_some() && !top {
                    return err(Diagnostic::error("a call that can fail must be the whole value: put it in its own `let`, then use the result").at(span));
                }
                self.call(ctx, path, args, otherwise.as_deref(), span, used)
            }
            AK::CallValue { callee, args, otherwise } => {
                if otherwise.is_some() && !top {
                    return err(Diagnostic::error("a call that can fail must be the whole value: put it in its own `let`, then use the result").at(span));
                }
                self.call_value(ctx, callee, args, otherwise.as_deref(), span, used)
            }
            AK::Field { .. } | AK::Index { .. } => {
                let place = self.place(ctx, e, PlaceUse::Read)?;
                let ty = place.ty.clone();
                Ok(Expr { kind: K::Read(place), ty, span })
            }
            AK::Verb { verb, amp, expr } => {
                if *amp {
                    self.amp_count += 1;
                }
                self.verb_expr(ctx, *verb, expr, span)
            }
            AK::Unary { op, expr } => self.unary(ctx, *op, expr, span),
            AK::Binary { op, op_span, lhs, rhs } => self.binary(ctx, op, *op_span, lhs, rhs, span),
            AK::Paren(inner) => {
                let mut v = self.expr(ctx, inner)?;
                v.span = span;
                Ok(v)
            }
            AK::List(items) => self.list_literal(ctx, items, span),
            AK::Conv { target, expr, otherwise } => {
                if otherwise.is_some() && !top {
                    return err(Diagnostic::error("a conversion that can fail must be the whole value: put it in its own `let`, then use the result").at(span));
                }
                self.conversion(ctx, target, expr, otherwise.as_deref(), span, used)
            }
            AK::Channel { elem, bound } => {
                let mut blanks = Vec::new();
                let et = self.items.resolve_type(elem, ctx.module, &mut blanks, false)?;
                let cap = match bound {
                    Some(b) => {
                        let (lo, hi) = (&b.0, &b.1);
                        let l = crate::resolve::int_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("a channel's bound is written as numbers: `channel[T][0, 100]`").at(lo.span)))?;
                        let h = crate::resolve::int_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("a channel's bound is written as numbers: `channel[T][0, 100]`").at(hi.span)))?;
                        if l < 0 || h < l {
                            return err(Diagnostic::error("a channel's bound is `[0, N]`: the low end 0, the high end how many values may wait").at(span));
                        }
                        Some((l as u64, h as u64))
                    }
                    None => None,
                };
                Ok(Expr { kind: K::Channel { elem: et.clone(), capacity: cap }, ty: Ty::Channel(Box::new(et)), span })
            }
            AK::FnLit(lit) => self.fn_literal(ctx, lit, span),
            AK::Pos(_) => err(Diagnostic::error("`pos` says a position, and one is only wanted where something is indexed or loaded: `'xs'[pos 3]`, `simd.load[lend 'xs', pos 'i']`").at(span)),
            AK::Spread { .. } => err(Diagnostic::error("`...` has no meaning here: it unpacks a list into a call's arguments — `sum[...'xs']`").at(span)),
            AK::SizeOf { align, ty } => {
                let mut blanks = Vec::new();
                let t = self.items.resolve_type(ty, ctx.module, &mut blanks, true)?;
                if t.contains_var() || !blanks.is_empty() {
                    return err(Diagnostic::error("a blank has no size until it is filled: `size.of` takes a concrete type").at(ty.span));
                }
                Ok(Expr { kind: K::SizeOf { align: *align, ty: t }, ty: Ty::u64(), span })
            }
            AK::Range { .. } => err(Diagnostic::error("`range[…]` is only decided as what a `for` runs over; as a value on its own it is not").at(span)),
        }
    }

    fn undeclared<T>(&self, ctx: &FnCtx, n: &ast::QName) -> Result<T> {
        if let Some(outer) = &ctx.captured_outer {
            if outer.iter().any(|(name, ..)| *name == n.name) {
                return err(Diagnostic::error(format!("'{}' is not declared inside this function: it sees nothing of the surrounding scope unless `with [lend '{}']` (or `copy` / `give`) says so", n.name, n.name)).at(n.span));
            }
        }
        if ctx.is_task {
            return err(Diagnostic::error(format!("'{}' is not declared inside this task: a task takes what it needs through `with [lend '{}']` (or `copy` / `give`)", n.name, n.name)).at(n.span));
        }
        err(Diagnostic::error(format!("'{}' is not declared", n.name)).at(n.span))
    }

    fn const_type(&mut self, id: hir::ConstId, span: Span) -> Result<Ty> {
        match &self.items.consts[id.0 as usize].ty {
            Some(t) => Ok(t.clone()),
            None => err(Diagnostic::error(format!("'{}' is used before its value is known: consts are evaluated in order, top to bottom", self.items.consts[id.0 as usize].name)).at(span)),
        }
    }

    /// A bare path as a value: a const, a function, an enum case, `list.empty`.
    fn path_value(&mut self, ctx: &mut FnCtx, path: &ast::Path, span: Span) -> Result<Expr> {
        if path.is(&["list", "empty"]) {
            let elem = self.fresh();
            return Ok(Expr { kind: K::ListEmpty, ty: Ty::Seq { elem: Box::new(elem), kind: SeqKind::Free(InferId(0)) }, span });
        }
        if path.is(&["map", "empty"]) {
            let k = self.fresh();
            let v = self.fresh();
            return Ok(Expr { kind: K::MapEmpty, ty: Ty::Map { key: Box::new(k), val: Box::new(v), order: MapOrder::Free(InferId(0)) }, span });
        }
        if path.is(&["list", "append"]) {
            return err(Diagnostic::error("`list.append` hands nothing back and cannot be used as a value").at(span));
        }
        if path.segs.len() == 1 && path.limb.is_none() {
            // A bare word: a const of this module? (consts are quoted; a bare word is never one.)
            let w = path.head();
            if crate::parser::is_keyword(w) {
                return err(Diagnostic::error(format!("`{}` is a keyword, not a value", w)).at(span));
            }
        }
        // An enum case without data: `Status.empty`.
        if path.segs.len() >= 2 {
            let prefix = ast::Path { limb: path.limb.clone(), segs: path.segs[..path.segs.len() - 1].to_vec(), span: path.span };
            if let Ok(ItemRef::Enum(id)) = self.items.lookup_path(&prefix, ctx.module) {
                return self.enum_case(ctx, id, path.last(), None, span);
            }
        }
        match self.items.lookup_path(path, ctx.module) {
            Ok(ItemRef::Fn(id)) => {
                let decl = &self.items.fns[id.0 as usize];
                if decl.op.is_some() {
                    return err(Diagnostic::error(format!("`{}` is an operator; it goes between two values and is not a value itself", decl.name)).at(span));
                }
                if !decl.blanks.is_empty() || decl.params.iter().any(|p| p.ty.is_none() || p.variadic) {
                    return err(Diagnostic::error(format!("`{}` has blanks, so it has no single type to hold: pass it where the blanks are filled, or write a version with the types", decl.name)).at(span));
                }
                let key = InstKey { fn_id: id, blanks: Vec::new(), bundle: None };
                let inst = self.instance(key);
                let sig = self.instances[inst.0 as usize].sig.clone();
                let ty = Ty::Fn(Box::new(FnTy { params: sig.params, ret: sig.ret, fails: sig.fails }));
                Ok(Expr { kind: K::FnRef(inst), ty, span })
            }
            Ok(ItemRef::ExternFn(_)) => err(Diagnostic::error(format!("`{}` is an extern function: it is called (under `nocheck`), and is not a value to hold", path.spelled())).at(span)),
            Ok(ItemRef::Struct(_)) => err(Diagnostic::error(format!("`{}` is a struct: a value of it is built with `{}[field = …]`", path.spelled(), path.spelled())).at(span)),
            Ok(ItemRef::Enum(id)) => err(Diagnostic::error(format!("`{}` is an enum: a value is one of its cases — `{}.{}`", path.spelled(), path.spelled(), self.items.enums[id.0 as usize].cases.first().map(|c| c.name.as_str()).unwrap_or("case"))).at(span)),
            Ok(other) => err(Diagnostic::error(format!("`{}` is a {}, not a value", path.spelled(), other.what())).at(span)),
            Err(e) => {
                if path.segs.len() == 1 && path.limb.is_none() {
                    return err(Diagnostic::error(format!("there is no function `{}` here: a bare name is the function as a value, and none by that name is declared in {}", path.head(), crate::resolve::describe_module(self.items.module_path(ctx.module)))).at(span));
                }
                Err(e)
            }
        }
    }

    // ------------------------------------------------------------ places

    /// A place: a local, a field of one, an element of one.
    pub fn place(&mut self, ctx: &mut FnCtx, e: &ast::Expr, use_: PlaceUse) -> Result<Place> {
        match &e.kind {
            AK::Name(n) => {
                let Some(id) = ctx.lookup(&n.name) else {
                    if let Some(w) = self.extern_let_named(&n.name, ctx) {
                        return self.extern_place(ctx, w, use_, e.span);
                    }
                    if self.items.modules[ctx.module.0 as usize].consts.contains_key(&n.name) {
                        return err(Diagnostic::error(format!("'{}' is a const and cannot be {}: it is a value known at compile time — nobody owns it, every use is a fresh value", n.name, match use_ { PlaceUse::Write => "assigned", PlaceUse::Give | PlaceUse::Copy | PlaceUse::Lend | PlaceUse::LendMut => "given, copied or lent", PlaceUse::Read => "used as a place" })).at(e.span));
                    }
                    return self.undeclared(ctx, n);
                };
                let local = &ctx.locals[id.0 as usize];
                self.check_place_use(ctx, id, use_, e.span, &n.name)?;
                Ok(Place { base: id, proj: Vec::new(), ty: local.ty.clone(), span: e.span })
            }
            AK::Field { base, field } => {
                let base_use = match use_ {
                    PlaceUse::Read => PlaceUse::Read,
                    PlaceUse::Write => PlaceUse::Write,
                    PlaceUse::Lend => PlaceUse::Read,
                    PlaceUse::LendMut => PlaceUse::Write,
                    PlaceUse::Give => PlaceUse::Give,
                    PlaceUse::Copy => PlaceUse::Read,
                };
                let mut p = self.place(ctx, base, base_use)?;
                let bt = self.resolve(&p.ty);
                let (sid, args) = match bt.base() {
                    Ty::Struct { id, args } => (*id, args.clone()),
                    Ty::Enum { .. } => return err(Diagnostic::error(format!("`.{}` reaches into a struct; an enum's data is reached by `match`", field.name)).at(field.span)),
                    other => return err(Diagnostic::error(format!("`.{}` reaches into a struct, and this is a `{}`", field.name, self.spell(other))).at(field.span)),
                };
                let info = &self.items.structs[sid.0 as usize];
                let Some(fi) = info.fields.iter().position(|f| f.name == field.name) else {
                    return err(Diagnostic::error(format!("`{}` has no field `{}`: its fields are {}", info.name, field.name, info.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", "))).at(field.span));
                };
                let f = &info.fields[fi];
                if use_ == PlaceUse::Write && f.immut {
                    return err(Diagnostic::error(format!("`{}` is an `immut` field of `{}` and cannot be written", field.name, info.name)).at(field.span));
                }
                let fty = f.ty.subst(&args);
                p.proj.push(Proj::Field(fi));
                p.ty = fty;
                p.span = e.span;
                Ok(p)
            }
            AK::Index { base, index } => {
                let base_use = match use_ {
                    PlaceUse::Write | PlaceUse::LendMut => PlaceUse::Write,
                    PlaceUse::Give => PlaceUse::Give,
                    _ => PlaceUse::Read,
                };
                let mut p = self.place(ctx, base, base_use)?;
                let bt = self.resolve(&p.ty);
                let elem = match bt.base() {
                    Ty::Seq { elem, .. } => (**elem).clone(),
                    Ty::Simd { elem, .. } => (**elem).clone(),
                    Ty::Str(_) => return err(Diagnostic::error("a text has no `[pos]`: it would not say which unit — the 3rd byte, point or grapheme? — use `std::str.at.<unit>` when std exists, or walk it with `str.each.<unit>`").at(e.span)),
                    Ty::Map { .. } => return err(Diagnostic::error("a map is looked up by key with `map.get[lend 'm', key] otherwise …`, not by position").at(e.span)),
                    Ty::Bundle(_) => return err(Diagnostic::error("a mixed bundle has no `[pos]`: walk it with `for`").at(e.span)),
                    other => return err(Diagnostic::error(format!("`[pos …]` indexes a list, and this is a `{}`", self.spell(other))).at(e.span)),
                };
                if use_ == PlaceUse::Give {
                    return err(Diagnostic::error("an element cannot be given away: it would leave a hole in the list — `list.remove[lend.mut 'xs', i]` hands the element back").at(e.span));
                }
                let i = self.expr(ctx, index)?;
                let it = self.resolve(&i.ty);
                if !it.is_int() {
                    return err(Diagnostic::error(format!("a position is an integer, and this is a `{}`", self.spell(&it))).at(index.span));
                }
                p.proj.push(Proj::Index(Box::new(i)));
                p.ty = elem;
                p.span = e.span;
                Ok(p)
            }
            AK::Paren(inner) => self.place(ctx, inner, use_),
            AK::Verb { .. } => err(Diagnostic::error("one verb per value").at(e.span)),
            _ => match use_ {
                PlaceUse::Write => err(Diagnostic::error("only a variable, a field or an element can be assigned to").at(e.span)),
                PlaceUse::Give => err(Diagnostic::error("`give` takes a variable (or a field): only something you hold can be given away").at(e.span)),
                PlaceUse::Copy => err(Diagnostic::error("`copy` takes a variable (or a field, or an element): a fresh value needs no copy").at(e.span)),
                _ => err(Diagnostic::error("`lend` takes a variable (or a field, or an element): a fresh value goes straight in, with no verb").at(e.span)),
            },
        }
    }

    fn extern_let_named(&self, name: &str, ctx: &FnCtx) -> Option<hir::ExternId> {
        let m = &self.items.modules[ctx.module.0 as usize];
        match m.items.get(&format!("'{}'", name)) {
            Some((ItemRef::ExternLet(id), _, _)) => Some(*id),
            _ => None,
        }
    }

    /// An `extern let` is a place of its own kind: a read or write is a nocheck site.
    fn extern_place(&mut self, ctx: &mut FnCtx, id: hir::ExternId, use_: PlaceUse, span: Span) -> Result<Place> {
        let info = self.items.externs[id.0 as usize].clone();
        if ctx.tier != Some(ast::Tier::Nocheck) {
            let what = if use_ == PlaceUse::Write { "writing" } else { "reading" };
            return err(Diagnostic::error(format!("{} C's variable '{}' is a nocheck site: nothing about C's memory can be proven — write `nocheck` in front of the statement", what, info.name)).at(span));
        }
        if use_ == PlaceUse::Write && !info.mutable {
            return err(Diagnostic::error(format!("'{}' is read-only: `extern let mut '{}': …` says C's variable may be written", info.name, info.name)).at(span));
        }
        if matches!(use_, PlaceUse::Give | PlaceUse::Copy | PlaceUse::Lend | PlaceUse::LendMut) {
            return err(Diagnostic::error(format!("'{}' is C's plain value: it copies by itself and takes no verb", info.name)).at(span));
        }
        // Model the extern as a synthetic local of kind Owned per use; the MIR builder maps it by name.
        let lid = ctx.declare(&format!("extern:{}", info.name), Local { name: info.name.clone(), ty: info.ret.clone(), mutability: None, kind: LocalKind::Owned, from_blank: false, span });
        ctx.locals[lid.0 as usize].name = format!("extern:{}", id.0);
        Ok(Place { base: lid, proj: Vec::new(), ty: info.ret.clone(), span })
    }

    fn check_place_use(&self, ctx: &FnCtx, id: hir::LocalId, use_: PlaceUse, span: Span, name: &str) -> Result<()> {
        let local = &ctx.locals[id.0 as usize];
        match use_ {
            PlaceUse::Write => {
                if local.mutability == Some(ast::Mutability::Immut) {
                    return err(Diagnostic::error(format!("'{}' is `immut` and cannot be assigned to", name)).at(span).why_at(local.span, "declared `immut` here"));
                }
                match local.kind {
                    LocalKind::Lent => return err(Diagnostic::error(format!("'{}' is lent (shared) and cannot be assigned to: it belongs to whoever lent it — `lent.mut` would allow it", name)).at(span).why_at(local.span, "lent here")),
                    LocalKind::LoopVar => return err(Diagnostic::error(format!("'{}' belongs to the loop and cannot be assigned: the header alone says what it is on every pass", name)).at(span).why_at(local.span, "the loop's variable")),
                    _ => {}
                }
            }
            PlaceUse::LendMut => {
                if local.mutability == Some(ast::Mutability::Immut) {
                    return err(Diagnostic::error(format!("'{}' is `immut`: it cannot be lent mutably", name)).at(span));
                }
                if local.kind == LocalKind::Lent {
                    return err(Diagnostic::error(format!("'{}' is lent (shared): it cannot be lent mutably on — `lent.mut` would allow it", name)).at(span));
                }
                if local.kind == LocalKind::LoopVar {
                    return err(Diagnostic::error(format!("'{}' is the loop's element: it cannot be changed through the loop", name)).at(span));
                }
            }
            PlaceUse::Give => {
                if matches!(local.kind, LocalKind::Lent | LocalKind::LentMut) {
                    return err(Diagnostic::error(format!("'{}' is lent: you cannot give away what you only borrowed", name)).at(span));
                }
                if local.kind == LocalKind::LoopVar {
                    return err(Diagnostic::error(format!("'{}' is the loop's element and belongs to the list: `copy` it", name)).at(span));
                }
            }
            _ => {}
        }
        Ok(())
    }

    // ------------------------------------------------------------ verbs

    /// `give`/`copy`/`lend`/`lend.mut` on a place, as a value.
    fn verb_expr(&mut self, ctx: &mut FnCtx, verb: Verb, operand: &ast::Expr, span: Span) -> Result<Expr> {
        // A verb on a call result or literal: nothing to transfer.
        match &operand.kind {
            AK::Name(_) | AK::Field { .. } | AK::Index { .. } | AK::Paren(_) => {}
            AK::Call { .. } | AK::CallValue { .. } | AK::Str(_) | AK::List(_) | AK::Conv { .. } => {
                return err(Diagnostic::error(format!("`{}` on a fresh value: a call's result or a literal has no owner yet, so it goes straight in with no verb", verb.spelled())).at(span));
            }
            _ => {}
        }
        let use_ = match verb {
            Verb::Give => PlaceUse::Give,
            Verb::Copy => PlaceUse::Copy,
            Verb::Lend => PlaceUse::Lend,
            Verb::LendMut => PlaceUse::LendMut,
        };
        let place = self.place(ctx, operand, use_)?;
        let ty = self.resolve(&place.ty);
        let from_blank = ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty();
        let name = self.place_name(ctx, &place);
        self.check_verb_on_type(&ty, verb, from_blank, span, &name)?;
        let kind = match verb {
            Verb::Give => K::Give(place),
            Verb::Copy => K::Copy(place),
            Verb::Lend => K::Lend { place, mutable: false },
            Verb::LendMut => K::Lend { place, mutable: true },
        };
        Ok(Expr { kind, ty, span })
    }

    pub fn place_name(&self, ctx: &FnCtx, p: &Place) -> String {
        let mut s = format!("'{}'", ctx.locals[p.base.0 as usize].name);
        for pr in &p.proj {
            match pr {
                Proj::Field(i) => {
                    // find the struct through the base type chain is expensive; keep the field index readable
                    s.push_str(&format!(".{}", self.field_name_at(ctx, p, *i)));
                }
                Proj::Index(_) => s.push_str("[pos …]"),
            }
        }
        s
    }

    fn field_name_at(&self, ctx: &FnCtx, p: &Place, idx: usize) -> String {
        // Walk the projections to find the struct that owns field `idx`.
        let mut t = self.resolve(&ctx.locals[p.base.0 as usize].ty);
        for pr in &p.proj {
            match pr {
                Proj::Field(i) => {
                    if let Ty::Struct { id, args } = t.base().clone() {
                        let info = &self.items.structs[id.0 as usize];
                        let name = info.fields[*i].name.clone();
                        if *i == idx {
                            return name;
                        }
                        t = info.fields[*i].ty.subst(&args);
                    }
                }
                Proj::Index(_) => {
                    if let Ty::Seq { elem, .. } = t.base().clone() {
                        t = (*elem).clone();
                    }
                }
            }
        }
        format!("field {}", idx)
    }

    /// The verb must fit the value: owned values take verbs, plain ones do not.
    pub fn check_verb_on_type(&self, ty: &Ty, verb: Verb, from_blank: bool, span: Span, name: &str) -> Result<()> {
        let ty = self.resolve(ty);
        if ty.is_owned() || from_blank || matches!(ty, Ty::Infer(_)) {
            return Ok(());
        }
        err(Diagnostic::error(format!("`{}` on {} is not possible: a `{}` is a plain value that owns nothing (no allocation behind it), so it copies by itself — write {} with no verb", verb.spelled(), name, self.spell(&ty), name)).at(span))
    }

    /// A value about to be stored (into a `let`, a field, an element, a
    /// parameter): an owned value must arrive fresh or with its verb.
    pub fn check_stored_value(&self, ctx: &FnCtx, v: &Expr, target: &Ty) -> Result<()> {
        let t = self.resolve(target);
        let _ = ctx;
        if let K::Read(place) = &v.kind {
            let pt = self.resolve(&place.ty);
            if pt.is_owned() {
                let name = self.place_name(ctx, place);
                let fb = ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty();
                if !fb {
                    return err(Diagnostic::error(format!("{} is an owned `{}`: say what happens to it — `give {}` (it moves), `copy {}` (a second one), or `lend {}` (a view)", name, self.spell(&pt), name, name, name)).at(v.span));
                }
            }
        }
        if let K::Lend { .. } = &v.kind {
            // Storing a lend: allowed into a `let` (a named view); the MIR checks its lifetime.
            let _ = t;
        }
        Ok(())
    }

    // ------------------------------------------------------------ stores and coercions

    /// Make `v` acceptable where a `target` is stored: a bare literal adopts
    /// the family; families must match; widths and bounds are Precog's
    /// check sites, not the checker's.
    pub fn expect_store(&mut self, ctx: &mut FnCtx, v: Expr, target: &Ty, what: &str) -> Result<Expr> {
        let _ = ctx;
        let t = self.resolve(target);
        let vt = self.resolve(&v.ty);
        // A bare integer literal takes its context's family; a *name* never
        // does (owner, 2026-09-22): Precog's freedom is over representation —
        // width, list or array, map order, which checks survive — never over
        // meaning, and 70000 in a `bin16` would silently become 69984.
        if let (K::Int(i), Ty::Bin(_)) = (&v.kind, t.base()) {
            return Ok(Expr { kind: K::Bin(i.to_string()), ty: t.base().clone(), span: v.span });
        }
        if let (K::Unary { op: UnOp::Neg, expr }, Ty::Bin(_)) = (&v.kind, t.base()) {
            if let K::Int(i) = &expr.kind {
                return Ok(Expr { kind: K::Bin(format!("-{}", i)), ty: t.base().clone(), span: v.span });
            }
        }
        if let (K::Char(c), Ty::Char(u)) = (&v.kind, t.base()) {
            let need = narrowest_unit(c);
            let fits = match u {
                Unit::Byte => need == Unit::Byte,
                Unit::Point => need != Unit::Grapheme,
                _ => true,
            };
            if !fits {
                let (n, what) = match need {
                    Unit::Point => (c.len(), "bytes"),
                    _ => (c.chars().count(), "code points"),
                };
                return err(Diagnostic::error(format!("`{}` is not one {}: it is {} {}, so it fits `char.{}`, not `{}`", c, match u { Unit::Byte => "byte", Unit::Point => "code point", _ => "grapheme" }, n, what, match need { Unit::Byte => "byte", Unit::Point => "point", _ => "grapheme" }, self.spell(&t))).at(v.span));
            }
        }
        // Arithmetic made only of literals has no family either, so in a
        // typed place it takes the place's: `let 'm': uint64 =
        // 9223372036854775807 + 1` is a `uint64` sum, 2^63, not an
        // `integer` one that overflows.
        let mut v = v;
        // `Ty::Infer` too: a shift, `not.bit` or wrapping operator of bare
        // numbers waits for its width from here (owner, 2026-09-23), and so
        // does every literal group inside it.
        if let (Ty::Int(IntW::Free(_)) | Ty::Infer(_), Ty::Int(IntW::Fixed { .. })) = (vt.base(), t.base()) {
            if literal_only(&v) {
                retype_literal_arith(&mut v, t.base());
            }
        }
        if self.unify(&vt, &t) {
            return Ok(v);
        }
        // Say it in the writer's terms.
        let msg = match (vt.base(), t.base()) {
            (Ty::Bin(_) | Ty::Deci(_), Ty::Int(_)) => {
                format!("{} is {} and this value is {}: there is no implicit conversion between numeric families — write `to.{}[…] otherwise …`, which fails `not_whole` rather than dropping the fraction", what, article(&self.spell(&t)), article(&self.spell(&vt)), conv_spelling(&self.spell(t.base())))
            }
            (Ty::Int(_), Ty::Bin(_)) | (Ty::Int(_), Ty::Deci(_)) | (Ty::Bin(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Bin(_)) => {
                format!("{} is {} and this value is {}: there is no implicit conversion between numeric families — write `to.{}[…]`", what, article(&self.spell(&t)), article(&self.spell(&vt)), conv_spelling(&self.spell(t.base())))
            }
            (Ty::Str(_), Ty::Int(_) | Ty::Bin(_)) => format!("{} is {} and this value is text: convert it first — `to.{}[…] otherwise …` (it can fail with `not_a_number`)", what, article(&self.spell(&t)), conv_spelling(&self.spell(t.base()))),
            (Ty::Int(_) | Ty::Bin(_), Ty::Str(_)) => format!("{} is text and this value is a `{}`: convert it first with `to.str.utf8[…]`", what, self.spell(&vt)),
            (Ty::Seq { .. }, Ty::Int(_)) if matches!(v.kind, K::List(ref l) if l.is_empty()) => format!("`[]` is an empty list, but {} is a `{}`", what, self.spell(&t)),
            (Ty::Seq { kind: SeqKind::List, .. }, Ty::Seq { kind: SeqKind::Array, .. }) => format!("{} is an `array` and this is a `list`", what),
            _ => format!("expected {} ({}), found {}", what.replace("the ", "a value of the "), self.spell(&t), article(&self.spell(&vt))),
        };
        err(Diagnostic::error(msg).at(v.span))
    }

    // ------------------------------------------------------------ operators

    fn unary(&mut self, ctx: &mut FnCtx, op: UnOp, operand: &ast::Expr, span: Span) -> Result<Expr> {
        let v = self.expr(ctx, operand)?;
        let vt = self.resolve(&v.ty);
        match op {
            UnOp::Neg => {
                if !vt.is_numeric() {
                    return err(Diagnostic::error(format!("`-` negates a number, and this is a `{}`", self.spell(&vt))).at(span));
                }
                if let Ty::Int(IntW::Fixed { signed: false, .. }) = vt.base() {
                    return err(Diagnostic::error(format!("`-` on a `{}`: an unsigned number is never negative", self.spell(&vt))).at(span));
                }
                let ty = vt.base().clone();
                Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span })
            }
            UnOp::Not => {
                self.expect_bool(&v, "`not`'s operand")?;
                Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty: Ty::Bool, span })
            }
            UnOp::NotBit => {
                match vt.base() {
                    Ty::Int(IntW::Fixed { .. }) => {}
                    Ty::Int(IntW::Free(_)) if literal_only(&v) => {
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), span, "not.bit".to_string()));
                        return Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty: t, span });
                    }
                    Ty::Int(IntW::Free(_)) => return err(Diagnostic::error("`not.bit` only knows a declared width: it flips every bit, so the value needs a type (`let 'x': uint8 = …`)").at(span)),
                    other => return err(Diagnostic::error(format!("`not.bit` works on integers, and this is a `{}`", self.spell(other))).at(span)),
                }
                let ty = vt.base().clone();
                Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span })
            }
        }
    }

    fn binary(&mut self, ctx: &mut FnCtx, op: &BinOp, op_span: Span, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        // A user operator takes its operands as a call would: lent by default, `give` by choice.
        if let BinOp::Custom(spelling) = op {
            return self.user_op(ctx, spelling, op_span, lhs, rhs, span);
        }
        // The language's operators may be defined for a user type: look first at the left operand.
        let l = self.expr(ctx, lhs)?;
        let lt = self.resolve(&l.ty);
        if matches!(lt.base(), Ty::Struct { .. } | Ty::Enum { .. }) && !matches!(op, BinOp::And | BinOp::Or) {
            if let Some(fn_id) = self.find_op(&op.spelled(), lt.base()) {
                let _ = l;
                return self.user_op_call(ctx, fn_id, &op.spelled(), op_span, lhs, rhs, span);
            }
            if matches!(op, BinOp::Eq | BinOp::Ne) {
                // structural equality
                let r = self.expr(ctx, rhs)?;
                let r = self.expect_operand(ctx, r, &lt, op, op_span)?;
                let l = self.as_operand(ctx, l)?;
                let r = self.as_operand(ctx, r)?;
                return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span });
            }
            return err(Diagnostic::error(format!("`{}` is not defined for `{}`: declare `op {} ['a': {}, 'b': {}] -> …` to give it one", op.spelled(), self.spell(&lt), op.spelled(), self.spell(&lt), self.spell(&lt))).at(op_span));
        }
        let r = self.expr(ctx, rhs)?;
        let rt = self.resolve(&r.ty);
        let l = self.as_operand(ctx, l)?;
        let r = self.as_operand(ctx, r)?;
        // Every operator a lane's type has works on a vector of them, lane by
        // lane (`simd.md`, owner: a); a scalar stands for every lane.
        if lt.is_simd() || rt.is_simd() {
            return self.simd_binary(op, op_span, l, r, &lt, &rt, span);
        }
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div | BinOp::Mod => {
                if lt.base() == &Ty::Duration || rt.base() == &Ty::Duration {
                    if !(lt.base() == &Ty::Duration && rt.base() == &Ty::Duration) || !matches!(op, BinOp::Add | BinOp::Sub) {
                        if matches!(op, BinOp::Mul) && (lt.is_int() || rt.is_int()) {
                            return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Duration, span });
                        }
                        return err(Diagnostic::error(format!("`{}` between a duration and a `{}` has no meaning: durations add, subtract, compare, and multiply by a whole number", op.spelled(), self.spell(if lt.base() == &Ty::Duration { &rt } else { &lt }))).at(op_span));
                    }
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Duration, span });
                }
                let (l, r, ty) = self.numeric_pair(ctx, l, r, op, op_span)?;
                if *op == BinOp::Pow && ty.is_int() {
                    if let K::Int(e) = &r.kind {
                        if *e < 0 {
                            return err(Diagnostic::error("a negative exponent on integers would produce a fraction: `xx` on integers takes a non-negative power").at(rhs.span));
                        }
                    }
                }
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
            }
            BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => {
                // The width is the left operand's declared type.
                match lt.base() {
                    Ty::Int(IntW::Fixed { .. }) => {}
                    // The width from the other side, or from where the
                    // value goes when both are bare numbers (owner,
                    // 2026-09-23).
                    Ty::Int(IntW::Free(_)) if matches!(rt.base(), Ty::Int(IntW::Fixed { .. })) => {
                        let ty = rt.base().clone();
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                    }
                    Ty::Int(IntW::Free(_)) if literal_only(&l) && literal_only(&r) => {
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), op_span, op.spelled().to_string()));
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                    }
                    Ty::Int(IntW::Free(_)) => return err(Diagnostic::error(format!("`{}` wraps at the type's size, so it only knows a declared one: give the left operand a type (`let 'x': uint32 = …`)", op.spelled())).at(op_span)),
                    other => return err(Diagnostic::error(format!("`{}` wraps an integer, and this is a `{}`: it only knows a declared integer width", op.spelled(), self.spell(other))).at(op_span)),
                }
                if !rt.is_int() {
                    return err(Diagnostic::error(format!("`{}` takes integers on both sides; the right side is a `{}`", op.spelled(), self.spell(&rt))).at(rhs.span));
                }
                let ty = lt.base().clone();
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
            }
            BinOp::Join => {
                if !lt.is_text() {
                    if lt.is_char() && rt.is_text() || lt.is_char() && rt.is_char() {
                        // a char joins as text
                    } else if lt.is_numeric() {
                        return err(Diagnostic::error(format!("`++` joins text, and its left side is {}: convert it first — `to.str.utf8[…] ++ …`", article(&self.spell(&lt)))).at(lhs.span));
                    } else {
                        return err(Diagnostic::error(format!("`++` joins text, and its left side is a `{}`", self.spell(&lt))).at(lhs.span));
                    }
                }
                if !rt.is_text() && !rt.is_char() {
                    if rt.is_numeric() {
                        return err(Diagnostic::error(format!("`++` joins text, and its right side is {}: convert it first — `… ++ to.str.utf8[…]`", article(&self.spell(&rt)))).at(rhs.span));
                    }
                    return err(Diagnostic::error(format!("`++` joins text, and its right side is a `{}`", self.spell(&rt))).at(rhs.span));
                }
                let enc = match (lt.base(), rt.base()) {
                    (Ty::Str(e), _) if !matches!(e, Enc::Free(_)) => *e,
                    (_, Ty::Str(e)) => *e,
                    _ => Enc::Free(InferId(0)),
                };
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Str(enc), span })
            }
            BinOp::Eq | BinOp::Ne => {
                let r = self.expect_operand(ctx, r, &lt, op, op_span)?;
                if matches!(lt.base(), Ty::Fn(_) | Ty::Ptr { .. } | Ty::Channel(_)) {
                    return err(Diagnostic::error(format!("`==` has no meaning for a `{}`", self.spell(&lt))).at(op_span));
                }
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                if lt.is_text() {
                    return err(Diagnostic::error(format!("`{}` on text has no meaning yet: byte order is never what anyone means, and a collation rule is std's — compare lengths or use a std function", op.spelled())).at(op_span));
                }
                if !lt.is_numeric() && lt.base() != &Ty::Duration && !lt.is_char() {
                    return err(Diagnostic::error(format!("`{}` compares numbers, and this is a `{}`", op.spelled(), self.spell(&lt))).at(op_span));
                }
                let r = self.expect_operand(ctx, r, &lt, op, op_span)?;
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::And | BinOp::Or => {
                self.expect_bool(&l, &format!("the left side of `{}`", op.spelled()))?;
                self.expect_bool(&r, &format!("the right side of `{}`", op.spelled()))?;
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr => {
                if !lt.is_int() || !rt.is_int() {
                    let bad = if !lt.is_int() { &lt } else { &rt };
                    return err(Diagnostic::error(format!("`{}` works on integers, and this is a `{}`", op.spelled(), self.spell(bad))).at(op_span));
                }
                // `shl` can push bits off the end, so it needs a width —
                // from any operand whose type is known, or, when every
                // operand is a bare number, from where the value goes
                // (owner, 2026-09-23).
                if matches!(op, BinOp::Shl) {
                    if let (Ty::Int(IntW::Free(_)), Ty::Int(IntW::Free(_))) = (lt.base(), rt.base()) {
                        if literal_only(&l) && literal_only(&r) {
                            let t = self.fresh();
                            self.widths_pending.push((t.clone(), op_span, "shl".to_string()));
                            return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                        }
                        return err(Diagnostic::error("`shl` only knows a declared width: shifting left can lose bits, so an operand needs a type").at(op_span));
                    }
                }
                let ty = if matches!(lt.base(), Ty::Int(IntW::Fixed { .. })) { lt.base().clone() } else { rt.base().clone() };
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
            }
            BinOp::Custom(_) => unreachable!(),
        }
    }

    /// A lane-wise operator. The rule is the scalar rule, N times.
    #[allow(clippy::too_many_arguments)]
    fn simd_binary(&mut self, op: &BinOp, op_span: Span, l: Expr, r: Expr, lt: &Ty, rt: &Ty, span: Span) -> Result<Expr> {
        let (lane, lanes) = match (lt.base(), rt.base()) {
            (Ty::Simd { elem, lanes }, Ty::Simd { elem: e2, lanes: l2 }) => {
                if !self.unify(elem, e2) {
                    return err(Diagnostic::error(format!("`{}` works lane by lane, and these hold different lanes: `{}` and `{}`", op.spelled(), self.spell(elem), self.spell(e2))).at(op_span));
                }
                if lanes != l2 && *lanes != 0 && *l2 != 0 {
                    return err(Diagnostic::error(format!("`{}` works lane by lane, and these are different lengths: {} and {}", op.spelled(), lanes, l2)).at(op_span));
                }
                ((**elem).clone(), (*lanes).max(*l2))
            }
            // A scalar broadcasts against a vector (the oracle's pick,
            // 2026-09-22; `simd.md` records it as something it had to choose).
            (Ty::Simd { elem, lanes }, scalar) | (scalar, Ty::Simd { elem, lanes }) => {
                let scalar = scalar.clone();
                if !self.unify(elem, &scalar) {
                    return err(Diagnostic::error(format!("`{}` puts a `{}` beside lanes of `{}`: a single value stands for every lane, so it has to be one of them", op.spelled(), self.spell(&scalar), self.spell(elem))).at(op_span));
                }
                ((**elem).clone(), *lanes)
            }
            _ => unreachable!(),
        };
        let lane_r = self.resolve(&lane);
        let ok = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div | BinOp::Mod | BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => lane_r.is_numeric(),
            BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr => lane_r.is_int(),
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => lane_r.is_numeric() || lane_r.base() == &Ty::Bool,
            BinOp::And | BinOp::Or => lane_r.base() == &Ty::Bool,
            _ => false,
        };
        if !ok {
            return err(Diagnostic::error(format!("`{}` is not an operator a lane of `{}` has, so a vector of them does not have it either", op.spelled(), self.spell(&lane_r))).at(op_span));
        }
        // A comparison gives one answer per lane, never a single `bool`.
        let ty = if op.is_comparison() { Ty::Simd { elem: Box::new(Ty::Bool), lanes } } else { Ty::Simd { elem: Box::new(lane), lanes } };
        Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
    }

    /// An operand read. A bare *owned* name in an operator is a view of it
    /// — nothing is handed over by comparing two texts. A plain value is
    /// simply read: it owns nothing, and a verb on one is an error
    /// everywhere else in the language, so it would be strange here.
    fn as_operand(&mut self, _ctx: &mut FnCtx, e: Expr) -> Result<Expr> {
        match e.kind {
            K::Read(place) => {
                let ty = e.ty;
                if self.resolve(&ty).is_owned() {
                    Ok(Expr { kind: K::Lend { place, mutable: false }, ty, span: e.span })
                } else {
                    Ok(Expr { kind: K::Read(place), ty, span: e.span })
                }
            }
            _ => Ok(e),
        }
    }

    /// Both sides of a numeric operator: one family; a literal adopts the other's.
    fn numeric_pair(&mut self, ctx: &mut FnCtx, l: Expr, r: Expr, op: &BinOp, op_span: Span) -> Result<(Expr, Expr, Ty)> {
        let lt = self.resolve(&l.ty);
        let rt = self.resolve(&r.ty);
        let (l, r) = match (lt.base(), rt.base()) {
            (Ty::Int(_), Ty::Bin(_)) if matches!(l.kind, K::Int(_)) => {
                let l = self.expect_store(ctx, l, &rt, "the operand")?;
                (l, r)
            }
            (Ty::Bin(_), Ty::Int(_)) if matches!(r.kind, K::Int(_)) => {
                let r = self.expect_store(ctx, r, &lt, "the operand")?;
                (l, r)
            }
            _ => (l, r),
        };
        // A bare whole-number literal takes the family of the other side
        // (`types.md`: "a bare numeric literal has no family of its own: it
        // takes the family of its context"). `'n' + 1` with `'n'` a `uint64`
        // is a `uint64` sum; joining the two made it an `integer`, which
        // cannot hold the upper half of a `uint64` and read it as negative.
        // A `:=` name whose width is still open is not a literal: that
        // choice stays Precog's.
        let lt = self.resolve(&l.ty);
        let rt = self.resolve(&r.ty);
        // A parenthesised group of literals — `(0 + 10702988103728947458)` —
        // has no family either, so it takes the other side's the same way,
        // every step of it.
        let (mut l, mut r) = (l, r);
        let adopt: Option<Ty> = match (lt.base(), rt.base()) {
            (Ty::Int(IntW::Free(_)), Ty::Int(w @ IntW::Fixed { .. })) if literal_only(&l) => {
                retype_literal_arith(&mut l, &Ty::Int(*w));
                Some(Ty::Int(*w))
            }
            (Ty::Int(w @ IntW::Fixed { .. }), Ty::Int(IntW::Free(_))) if literal_only(&r) => {
                retype_literal_arith(&mut r, &Ty::Int(*w));
                Some(Ty::Int(*w))
            }
            _ => None,
        };
        // The literal keeps its value — a literal too big for the family is
        // then a narrowing Precog refuses as certain, like any other — but a
        // negative one cannot join an unsigned family at all.
        if let Some(Ty::Int(IntW::Fixed { signed: false, .. })) = &adopt {
            let lit = if literal_only(&l) && matches!(lt.base(), Ty::Int(IntW::Free(_))) { &l } else { &r };
            if is_int_literal(lit) && (matches!(lit.kind, K::Unary { op: crate::ast::UnOp::Neg, .. }) || matches!(lit.kind, K::Int(v) if v < 0)) {
                let fam = adopt.as_ref().map(|t| self.spell(t)).unwrap_or_default();
                return err(Diagnostic::error(format!("a negative number beside a `{}`: an unsigned number is never negative, and a bare number takes the family of the other side", fam)).at(lit.span));
            }
        }
        if !lt.is_numeric() {
            return err(Diagnostic::error(format!("`{}` works on numbers, and its left side is a `{}`", op.spelled(), self.spell(&lt))).at(l.span));
        }
        if !rt.is_numeric() {
            return err(Diagnostic::error(format!("`{}` works on numbers, and its right side is a `{}`", op.spelled(), self.spell(&rt))).at(r.span));
        }
        let ty = match (lt.base(), rt.base()) {
            _ if adopt.is_some() => adopt.clone().unwrap(),
            (Ty::Int(a), Ty::Int(b)) => Ty::Int(join_int(*a, *b)),
            (Ty::Bin(a), Ty::Bin(b)) => Ty::Bin(join_bin(*a, *b)),
            (Ty::Deci(a), Ty::Deci(b)) => Ty::Deci((*a).max(*b)),
            _ => return err(Diagnostic::error(format!("`{}` between a `{}` and a `{}`: there is no implicit conversion between numeric families — convert one side with `to.{}[…]`", op.spelled(), self.spell(&lt), self.spell(&rt), self.spell(lt.base()))).at(op_span)),
        };
        Ok((l, r, ty))
    }

    /// The right side of a comparison must be the left's family.
    fn expect_operand(&mut self, ctx: &mut FnCtx, r: Expr, lt: &Ty, op: &BinOp, op_span: Span) -> Result<Expr> {
        let rt = self.resolve(&r.ty);
        if lt.is_numeric() && rt.is_numeric() {
            let (_, r2, _) = self.numeric_pair(ctx, Expr { kind: K::Nothing, ty: lt.clone(), span: op_span }, r, op, op_span)?;
            return Ok(r2);
        }
        if lt.is_char() && rt.is_char() {
            return Ok(r);
        }
        if self.unify(lt, &rt) {
            return Ok(r);
        }
        err(Diagnostic::error(format!("`{}` compares two values of one type: the left is a `{}`, the right a `{}`", op.spelled(), self.spell(lt), self.spell(&rt))).at(op_span))
    }

    fn user_op(&mut self, ctx: &mut FnCtx, spelling: &str, op_span: Span, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        let l = self.expr(ctx, lhs)?;
        let lt = self.resolve(&l.ty);
        let Some(fn_id) = self.find_op(spelling, lt.base()) else {
            let hint = match spelling {
                "*" => "multiply is `x`",
                "^" => "power is `xx`",
                "%" => "remainder is `mod`",
                "&" | "|" | "!" => "the boolean operators are `and`, `or`, `not`; the bitwise ones `and.bit`, `or.bit`, `not.bit`; a lend is `lend 'x'`",
                "&+" | "&-" | "+%" | "-%" | "*%" => "wrapping arithmetic is `+~`, `-~`, `x~`, `xx~`",
                _ => "an operator between two values is one the language owns (`+`, `x`, `mod`, …) or one declared with `op`",
            };
            return err(Diagnostic::error(format!("`{}` is not an operator here: {}", spelling, hint)).at(op_span));
        };
        self.user_op_call(ctx, fn_id, spelling, op_span, lhs, rhs, span)
    }

    /// A user `op` applied: operands lent by default, `give` when the op says so.
    fn user_op_call(&mut self, ctx: &mut FnCtx, fn_id: FnId, spelling: &str, op_span: Span, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        let decl = self.items.fns[fn_id.0 as usize].clone();
        let key = InstKey { fn_id, blanks: Vec::new(), bundle: None };
        let inst = self.instance(key);
        let sig = self.instances[inst.0 as usize].sig.clone();
        let mut operands = Vec::new();
        for (i, (side, (kind, pty))) in [lhs, rhs].into_iter().zip(&sig.params).enumerate() {
            let pdecl = &decl.params[i];
            let gives = *kind == ParamKind::Owned && pdecl.ty.as_ref().map(|t| self.resolve(t).is_owned()).unwrap_or(false);
            let arg = match &side.kind {
                AK::Verb { verb: Verb::Give, expr, .. } => {
                    if !gives {
                        return err(Diagnostic::error(format!("`{}` lends its operand: `give` is not wanted here", spelling)).at(side.span));
                    }
                    let place = self.place(ctx, expr, PlaceUse::Give)?;
                    Arg::Give(place)
                }
                AK::Verb { verb, .. } => return err(Diagnostic::error(format!("an operand takes no `{}`: operands are lent unless the operator says `give`", verb.spelled())).at(side.span)),
                _ => {
                    let v = self.expr(ctx, side)?;
                    let vt = self.resolve(&v.ty);
                    if gives {
                        match &v.kind {
                            K::Read(_) => {
                                let n = pdecl.name.clone();
                                return err(Diagnostic::error(format!("`{}` takes '{}' by value: write `give {}` so the value moves in", spelling, n, self.expr_name(ctx, &v))).at(side.span));
                            }
                            _ => Arg::Value(v),
                        }
                    } else {
                        let v = self.expect_store(ctx, v, pty, "the operand")?;
                        match v.kind {
                            K::Read(place) if vt.is_owned() => Arg::Lend { place, mutable: false },
                            _ => Arg::Value(v),
                        }
                    }
                }
            };
            operands.push(arg);
        }
        let rhs_arg = operands.pop().unwrap();
        let lhs_arg = operands.pop().unwrap();
        // types must match the declared operands
        let _ = op_span;
        Ok(Expr { kind: K::UserOp { inst, lhs: Box::new(lhs_arg), rhs: Box::new(rhs_arg) }, ty: sig.ret, span })
    }

    fn expr_name(&self, ctx: &FnCtx, e: &Expr) -> String {
        match &e.kind {
            K::Read(p) | K::Lend { place: p, .. } | K::Copy(p) | K::Give(p) => self.place_name(ctx, p),
            K::Local(id) => format!("'{}'", ctx.locals[id.0 as usize].name),
            _ => "it".into(),
        }
    }

    // ------------------------------------------------------------ literals

    fn list_literal(&mut self, ctx: &mut FnCtx, items: &[ast::Expr], span: Span) -> Result<Expr> {
        if items.is_empty() {
            // `[]` is a value only where a declared type says list; the store decides.
            let elem = self.fresh();
            return Ok(Expr { kind: K::List(Vec::new()), ty: Ty::Seq { elem: Box::new(elem), kind: SeqKind::Free(InferId(0)) }, span });
        }
        let mut out = Vec::new();
        let mut elem: Option<Ty> = None;
        for it in items {
            let v = self.expr(ctx, it)?;
            let v = match &elem {
                Some(t) => {
                    let t = t.clone();
                    self.expect_store(ctx, v, &t, "the list's element type")?
                }
                None => v,
            };
            let vt = self.resolve(&v.ty);
            if elem.is_none() {
                elem = Some(vt.clone());
            } else if let Some(Ty::Int(a)) = elem.as_ref().map(|t| t.base().clone()) {
                if let Ty::Int(b) = vt.base() {
                    elem = Some(Ty::Int(join_int(a, *b)));
                }
            }
            self.check_stored_value(ctx, &v, &vt)?;
            out.push(self.value_arg(v));
        }
        let elem = elem.unwrap();
        Ok(Expr { kind: K::List(out), ty: Ty::Seq { elem: Box::new(elem), kind: SeqKind::Free(InferId(0)) }, span })
    }

    /// A value that is stored somewhere: a verb'd place becomes the matching arg.
    pub fn value_arg(&self, v: Expr) -> Arg {
        match v.kind {
            K::Give(p) => Arg::Give(p),
            K::Copy(p) => Arg::Copy(p),
            K::Lend { place, mutable } => Arg::Lend { place, mutable },
            _ => Arg::Value(v),
        }
    }

    fn enum_case(&mut self, ctx: &mut FnCtx, id: EnumId, case: &ast::Ident, payload: Option<&[AArg]>, span: Span) -> Result<Expr> {
        let info = self.items.enums[id.0 as usize].clone();
        let Some(ci) = info.cases.iter().position(|c| c.name == case.name) else {
            return err(Diagnostic::error(format!("`{}` has no case `{}`: its cases are {}", info.name, case.name, info.cases.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", "))).at(case.span));
        };
        let args: Vec<Ty> = info.blanks.iter().map(|_| self.fresh()).collect();
        let expected = info.cases[ci].payload.as_ref().map(|t| t.subst(&args));
        let payload = match (payload, expected) {
            (None, None) => None,
            (Some(a), Some(t)) => {
                if a.len() != 1 {
                    return err(Diagnostic::error(format!("`{}.{}` carries one value", info.name, case.name)).at(span));
                }
                let AArg::Positional(e) = &a[0] else {
                    return err(Diagnostic::error(format!("`{}.{}` carries one value, not a field", info.name, case.name)).at(span));
                };
                let v = self.expr(ctx, e)?;
                let v = self.expect_store(ctx, v, &t, &format!("`{}`'s data", case.name))?;
                self.check_stored_value(ctx, &v, &t)?;
                Some(Box::new(self.value_arg(v)))
            }
            (None, Some(t)) => return err(Diagnostic::error(format!("`{}.{}` carries a `{}`: write `{}.{}[…]` with the value", info.name, case.name, self.spell(&t), info.name, case.name)).at(span)),
            (Some(_), None) => return err(Diagnostic::error(format!("`{}.{}` carries nothing; write it bare — `{}.{}`", info.name, case.name, info.name, case.name)).at(span)),
        };
        Ok(Expr { kind: K::EnumCase { id, case: ci, payload }, ty: Ty::Enum { id, args }, span })
    }

    fn construct(&mut self, ctx: &mut FnCtx, id: StructId, args: &[AArg], span: Span) -> Result<Expr> {
        let info = self.items.structs[id.0 as usize].clone();
        let targs: Vec<Ty> = info.blanks.iter().map(|_| self.fresh()).collect();
        let mut given: Vec<Option<Arg>> = vec![None; info.fields.len()];
        for a in args {
            match a {
                AArg::Named(name, e) => {
                    let Some(fi) = info.fields.iter().position(|f| f.name == name.name) else {
                        return err(Diagnostic::error(format!("`{}` has no field `{}`: its fields are {}", info.name, name.name, info.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", "))).at(name.span));
                    };
                    if given[fi].is_some() {
                        return err(Diagnostic::error(format!("field `{}` is given twice", name.name)).at(name.span));
                    }
                    let t = info.fields[fi].ty.subst(&targs);
                    let v = self.expr(ctx, e)?;
                    let v = self.expect_store(ctx, v, &t, &format!("field `{}`", name.name))?;
                    self.check_stored_value(ctx, &v, &t)?;
                    given[fi] = Some(self.value_arg(v));
                }
                AArg::Positional(e) => return err(Diagnostic::error(format!("a struct is built by field name: `{}[{} = …]`", info.name, info.fields.first().map(|f| f.name.as_str()).unwrap_or("field"))).at(e.span)),
                AArg::Pair(k, _) => return err(Diagnostic::error(format!("a struct is built by field name: `{}[{} = …]`", info.name, info.fields.first().map(|f| f.name.as_str()).unwrap_or("field"))).at(k.span)),
            }
        }
        let missing: Vec<&str> = info.fields.iter().enumerate().filter(|(i, _)| given[*i].is_none()).map(|(_, f)| f.name.as_str()).collect();
        if !missing.is_empty() {
            return err(Diagnostic::error(format!("`{}` is missing {}: every field must be given — there are no zero values", info.name, missing.iter().map(|m| format!("`{}`", m)).collect::<Vec<_>>().join(", "))).at(span));
        }
        let fields = given.into_iter().map(|g| g.unwrap()).collect();
        Ok(Expr { kind: K::Construct { id, fields }, ty: Ty::Struct { id, args: targs }, span })
    }

    fn map_literal(&mut self, ctx: &mut FnCtx, args: &[AArg], span: Span) -> Result<Expr> {
        if args.is_empty() {
            return err(Diagnostic::error("an empty map is `map.empty`; `map[]` builds nothing").at(span));
        }
        let kt = self.fresh();
        let vt = self.fresh();
        let mut pairs = Vec::new();
        let mut seen_keys: Vec<Expr> = Vec::new();
        for a in args {
            let AArg::Pair(k, v) = a else {
                return err(Diagnostic::error("a map literal pairs each key with a value: `map[\"ann\" = 5551, \"bob\" = 5552]`").at(a.span()));
            };
            let kv = self.expr(ctx, k)?;
            let kv = self.expect_store(ctx, kv, &kt, "the key type")?;
            self.items.check_key_type(&self.resolve(&kv.ty), k.span)?;
            self.check_stored_value(ctx, &kv, &kt)?;
            for prev in &seen_keys {
                if same_literal(prev, &kv) {
                    return err(Diagnostic::error("this key is written twice in the literal").at(k.span));
                }
            }
            seen_keys.push(kv.clone());
            let vv = self.expr(ctx, v)?;
            let vv = self.expect_store(ctx, vv, &vt, "the value type")?;
            self.check_stored_value(ctx, &vv, &vt)?;
            pairs.push((self.value_arg(kv), self.value_arg(vv)));
        }
        Ok(Expr { kind: K::MapLit(pairs), ty: Ty::Map { key: Box::new(kt), val: Box::new(vt), order: MapOrder::Free(InferId(0)) }, span })
    }

    // ------------------------------------------------------------ calls

    fn call(&mut self, ctx: &mut FnCtx, path: &ast::Path, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        if self.std_stubs && path.limb.as_ref().map(|l| l.name == "std").unwrap_or(false) {
            return self.std_stub(ctx, path, args, otherwise, span, used);
        }
        // Language operations first.
        if path.limb.is_none() {
            if let Some(b) = self.builtin_head(path, span)? {
                return self.builtin_call(ctx, b, path, args, otherwise, span, used);
            }
        }
        // An enum case with data: `Status.claimed["x"]`.
        if path.segs.len() >= 2 && path.limb.is_none() {
            let prefix = ast::Path { limb: None, segs: path.segs[..path.segs.len() - 1].to_vec(), span: path.span };
            if let Ok(ItemRef::Enum(id)) = self.items.lookup_path(&prefix, ctx.module) {
                if otherwise.is_some() {
                    return err(Diagnostic::error("building an enum value cannot fail; drop the `otherwise`").at(span));
                }
                return self.enum_case(ctx, id, path.last(), Some(args), span);
            }
        }
        let item = match self.items.lookup_path(path, ctx.module) {
            Ok(i) => i,
            Err(e) => {
                if path.segs.len() == 1 && path.limb.is_none() && self.items.ops.contains_key(path.head()) {
                    return err(Diagnostic::error(format!("`{}` is an operator; it goes between two values: `'a' {} 'b'`, never `{}[…]`", path.head(), path.head(), path.head())).at(span));
                }
                if path.segs.len() == 1 && path.limb.is_none() && args.iter().any(|a| matches!(a, AArg::Named(..))) {
                    return err(Diagnostic::error(format!("there is no struct named `{}` here", path.head())).at(path.span));
                }
                return Err(e);
            }
        };
        match item {
            ItemRef::Struct(id) => {
                if otherwise.is_some() {
                    return err(Diagnostic::error("building a struct value cannot fail; drop the `otherwise`").at(span));
                }
                self.construct(ctx, id, args, span)
            }
            ItemRef::Fn(id) => self.fn_call(ctx, id, args, otherwise, span, used, path),
            ItemRef::ExternFn(id) => self.extern_call(ctx, id, args, otherwise, span, used),
            ItemRef::Enum(_) => err(Diagnostic::error(format!("`{}` is an enum: a value is one of its cases — `{}.case` or `{}.case[data]`", path.spelled(), path.spelled(), path.spelled())).at(span)),
            other => err(Diagnostic::error(format!("`{}` is a {}, not a function", path.spelled(), other.what())).at(span)),
        }
    }

    fn fn_call(&mut self, ctx: &mut FnCtx, id: FnId, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool, path: &ast::Path) -> Result<Expr> {
        let decl = self.items.fns[id.0 as usize].clone();
        if decl.op.is_some() {
            return err(Diagnostic::error(format!("`{}` is an operator; it goes between two values: `'a' {} 'b'`, never `{}[…]`", decl.name, decl.name, decl.name)).at(span));
        }
        // Blanks: fresh inference variables, filled by the arguments.
        let named: Vec<Ty> = decl.blanks.iter().map(|_| self.fresh()).collect();
        let mut implicit: Vec<Ty> = Vec::new();
        let mut param_tys: Vec<(ParamKind, Ty, bool)> = Vec::new(); // (kind, type, variadic)
        for p in &decl.params {
            let t = match &p.ty {
                Some(t) => t.subst(&named),
                None => {
                    let f = self.fresh();
                    implicit.push(f.clone());
                    f
                }
            };
            param_tys.push((p.kind, t, p.variadic));
        }
        let fixed = param_tys.iter().filter(|p| !p.2).count();
        let has_variadic = param_tys.iter().any(|p| p.2);
        // Unpack the arguments: spreads count as their length.
        let flat = self.flatten_args(ctx, args, span)?;
        if !has_variadic && flat.len() != fixed {
            return err(Diagnostic::error(format!("`{}` takes {} argument{}, and {} {} given", decl.name, fixed, if fixed == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).at(span).why_at(decl.name_span, format!("`{}` is declared here", decl.name)));
        }
        if has_variadic && flat.len() < fixed {
            return err(Diagnostic::error(format!("`{}` takes at least {} argument{}, and {} {} given", decl.name, fixed, if fixed == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).at(span));
        }
        let mut hargs = Vec::new();
        let mut bundle_tys = Vec::new();
        for (i, fa) in flat.into_iter().enumerate() {
            let (kind, pty, variadic, pname) = if i < fixed {
                let p = &param_tys[i];
                (p.0, p.1.clone(), false, decl.params[i].name.clone())
            } else {
                let p = param_tys.last().unwrap();
                (p.0, p.1.clone(), true, decl.params.last().unwrap().name.clone())
            };
            let elem_ty = if variadic {
                // an untyped variadic keeps each argument's own type
                match &decl.params.last().unwrap().ty {
                    Some(_) => pty.clone(),
                    None => self.fresh(),
                }
            } else {
                pty.clone()
            };
            let from_blank = i < fixed && decl.params[i].from_blank && decl.params[i].ty.as_ref().map(|t| matches!(t, Ty::Var(_))).unwrap_or(false);
            let filled_before = from_blank && !matches!(self.resolve(&pty), Ty::Infer(_));
            let a = match self.argument(ctx, fa, kind, &elem_ty, &decl.name, &pname, variadic) {
                Ok(a) => a,
                Err(e) if filled_before => {
                    let blank = match &decl.params[i].ty { Some(Ty::Var(b)) => decl.blanks[*b as usize].0.clone(), _ => "T".into() };
                    return err(Diagnostic::error(format!("one blank, one type: `<{}>` is filled by an earlier argument as `{}`, and '{}' would make it something else", blank, self.spell(&pty), pname)).at(e.span.unwrap_or(span)));
                }
                Err(e) => return Err(e),
            };
            if variadic {
                match &a {
                    Arg::Spread { expr, count } => {
                        let lt = self.resolve(&self.arg_type(expr));
                        let et = self.elem_type_of(&lt, span, "unpacking")?;
                        match self.resolve(&et) {
                            Ty::Bundle(ts) => bundle_tys.extend(ts),
                            t => {
                                for _ in 0..*count {
                                    bundle_tys.push(t.clone());
                                }
                            }
                        }
                    }
                    _ => bundle_tys.push(self.resolve(&self.arg_type(&a))),
                }
            }
            hargs.push(a);
        }
        // Every blank must be filled by now.
        let mut blank_tys = Vec::new();
        for (i, t) in named.iter().enumerate() {
            let r = self.resolve(t);
            if matches!(r, Ty::Infer(_)) {
                return err(Diagnostic::error(format!("the blank `<{}>` of `{}` is not filled by this call: no argument says what it is", decl.blanks[i].0, decl.name)).at(span));
            }
            // interface bound
            if let Some(iface) = decl.blanks[i].1 {
                if let Err(reason) = self.satisfies(&r, iface) {
                    let iname = self.items.ifaces[iface.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` asks for a `<{}: {}>`", self.spell(&r), iname, reason, decl.name, decl.blanks[i].0, iname)).at(span));
                }
            }
            blank_tys.push(r);
        }
        for t in &implicit {
            let r = self.resolve(t);
            blank_tys.push(r);
        }
        let key = InstKey { fn_id: id, blanks: blank_tys, bundle: if has_variadic { Some(bundle_tys) } else { None } };
        let inst = self.instance(key);
        let sig = self.instances[inst.0 as usize].sig.clone();
        let _ = path;
        let ow = self.call_otherwise(ctx, otherwise, &sig.fails, &sig.ret, used, span, &decl.name)?;
        Ok(Expr { kind: K::Call { callee: Callee::Fn(inst), args: hargs, otherwise: ow, fails: sig.fails.clone() }, ty: sig.ret, span })
    }

    fn call_otherwise(&mut self, ctx: &mut FnCtx, otherwise: Option<&ast::Otherwise>, fails: &[(String, Option<Ty>)], ret: &Ty, used: bool, span: Span, name: &str) -> Result<Option<Box<hir::Otherwise>>> {
        match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error(format!("`{}` cannot fail; drop the `otherwise`", name)).at(span));
                }
                Ok(Some(Box::new(self.otherwise(ctx, o, fails, ret, used, span)?)))
            }
            None => {
                if !fails.is_empty() {
                    let list = fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", ");
                    return err(Diagnostic::error(format!("`{}` can fail ({}): say what happens — `otherwise` a value, `otherwise return …`, `otherwise fail`, or `otherwise {{ {} {{ … }} }}` arms", name, list, fails[0].0)).at(span));
                }
                Ok(None)
            }
        }
    }

    /// The part after `otherwise`, checked against the failures it handles.
    pub fn otherwise(&mut self, ctx: &mut FnCtx, o: &ast::Otherwise, fails: &[(String, Option<Ty>)], ret: &Ty, used: bool, span: Span) -> Result<hir::Otherwise> {
        match o {
            ast::Otherwise::Value(e) => {
                let v = self.expr(ctx, e)?;
                let vt = self.resolve(&v.ty);
                if used || vt != Ty::Nothing {
                    // a leaving call (`std::exit[…]`) returns nothing and is fine either way
                    if vt != Ty::Nothing {
                        let v = self.expect_store(ctx, v, ret, "the fallback value")?;
                        self.check_stored_value(ctx, &v, ret)?;
                        return Ok(hir::Otherwise::Value(v));
                    }
                }
                Ok(hir::Otherwise::Value(v))
            }
            ast::Otherwise::Leave(s) => {
                let st = self.stmt(ctx, s)?;
                Ok(hir::Otherwise::Leave(Box::new(st)))
            }
            ast::Otherwise::PassUp(sp) => {
                if ctx.is_task {
                    // A task collects what it passes up; its `otherwise` handles them.
                    for (n, t) in fails {
                        if !ctx.fails.iter().any(|(x, _)| x == n) {
                            ctx.fails.push((n.clone(), t.clone()));
                        }
                    }
                    return Ok(hir::Otherwise::PassUp);
                }
                if ctx.is_main {
                    return err(Diagnostic::error("`MAIN` cannot fail, so it cannot pass a failure up: handle it here, or exit").at(*sp));
                }
                for (n, _) in fails {
                    if !ctx.fails.iter().any(|(x, _)| x == n) {
                        if ctx.fails.is_empty() {
                            return err(Diagnostic::error(format!("`{}` cannot fail, so it cannot pass `{}` up: add `-> … or fail[{}]` to its signature, or handle it here", ctx.name, n, n)).at(*sp));
                        }
                        return err(Diagnostic::error(format!("`{}` does not list `{}` among its failures, so `otherwise fail` cannot pass it up: its signature says fail[{}] — add `{}` there, or handle it here", ctx.name, n, ctx.fails.iter().map(|(x, _)| x.clone()).collect::<Vec<_>>().join(", "), n)).at(*sp));
                    }
                }
                Ok(hir::Otherwise::PassUp)
            }
            ast::Otherwise::Arms(arms, arms_span) => {
                let mut out = Vec::new();
                let mut covered = vec![false; fails.len()];
                // Names first: an unknown or missing reason is the message, before any body is read.
                for arm in arms {
                    if let Some(fi) = fails.iter().position(|(n, _)| *n == arm.name.name) {
                        covered[fi] = true;
                    } else {
                        return err(Diagnostic::error(format!("this call never fails with `{}`: it fails with {}", arm.name.name, fails.iter().map(|(n, _)| format!("`{}`", n)).collect::<Vec<_>>().join(", "))).at(arm.name.span));
                    }
                }
                let missing: Vec<&str> = fails.iter().enumerate().filter(|(i, _)| !covered[*i]).map(|(_, (n, _))| n.as_str()).collect();
                if !missing.is_empty() {
                    return err(Diagnostic::error(format!("this call can also fail with {}: every reason needs an arm", missing.iter().map(|m| format!("`{}`", m)).collect::<Vec<_>>().join(", "))).at(*arms_span));
                }
                let mut covered = vec![false; fails.len()];
                for arm in arms {
                    let Some(fi) = fails.iter().position(|(n, _)| *n == arm.name.name) else {
                        return err(Diagnostic::error(format!("this call never fails with `{}`: it fails with {}", arm.name.name, fails.iter().map(|(n, _)| format!("`{}`", n)).collect::<Vec<_>>().join(", "))).at(arm.name.span));
                    };
                    if covered[fi] {
                        return err(Diagnostic::error(format!("`{}` is handled twice", arm.name.name)).at(arm.name.span));
                    }
                    covered[fi] = true;
                    ctx.push_scope();
                    let bind = match (&arm.bind, &fails[fi].1) {
                        (None, None) => None,
                        (Some(b), Some(t)) => Some(ctx.declare(&b.name, Local { name: b.name.clone(), ty: t.clone(), mutability: None, kind: LocalKind::Owned, from_blank: false, span: b.span })),
                        (Some(b), None) => return err(Diagnostic::error(format!("`{}` carries nothing, so there is no value to name '{}'", arm.name.name, b.name)).at(b.span)),
                        (None, Some(t)) => return err(Diagnostic::error(format!("`{}` carries a `{}`: name it — `{} 'value' {{ … }}`", arm.name.name, self.spell(t), arm.name.name)).at(arm.name.span)),
                    };
                    let body = self.block(ctx, &arm.body)?;
                    ctx.pop_scope();
                    if used && !super::stmt::leaves_block(&body) {
                        return err(Diagnostic::error(format!("the `{}` arm must leave (`return`, `fail`, `break`, `continue` or `std::exit`): the call's value is used, and this arm gives it nothing", arm.name.name)).at(arm.body.span));
                    }
                    out.push(hir::FailArm { index: fi, bind, body });
                }
                let _ = span;
                Ok(hir::Otherwise::Arms(out))
            }
        }
    }

    /// One flattened argument: what was written, with its verb, before matching a parameter.
    fn flatten_args(&mut self, ctx: &mut FnCtx, args: &[AArg], span: Span) -> Result<Vec<FlatArg>> {
        let mut out = Vec::new();
        for a in args {
            match a {
                AArg::Positional(e) => match &e.kind {
                    AK::Spread { verb, expr } => {
                        let (count, elem_verb) = self.spread_count(ctx, expr, *verb, e.span)?;
                        out.push(FlatArg::Spread { expr: (**expr).clone(), verb: elem_verb, count, span: e.span });
                    }
                    _ => out.push(FlatArg::One(e.clone())),
                },
                AArg::Named(n, _) => return err(Diagnostic::error(format!("`{} = …` names a field, but this is a call: arguments are values in order", n.name)).at(n.span)),
                AArg::Pair(k, _) => return err(Diagnostic::error("`key = value` pairs belong in a `map[…]` literal, not a call").at(k.span)),
            }
        }
        let _ = span;
        Ok(out)
    }

    /// How many arguments a spread stands for: known from a literal, a bound, a bundle.
    fn spread_count(&mut self, ctx: &mut FnCtx, expr: &ast::Expr, verb: Option<Verb>, span: Span) -> Result<(usize, Option<Verb>)> {
        let place = self.place(ctx, expr, PlaceUse::Read)?;
        let t = self.resolve(&place.ty);
        let count = match &t {
            Ty::Bundle(ts) => ts.len(),
            Ty::Bounded { bound: Bound::Len { hi, .. }, inner } if inner.is_seq() => *hi as usize,
            Ty::Seq { .. } => {
                // A plain list: its length must be known here — a literal list held by a local.
                let local = &ctx.locals[place.base.0 as usize];
                match self.known_length(ctx, &place) {
                    Some(n) => n,
                    None => return err(Diagnostic::error(format!("the length of {} is not known here, so `...` cannot unpack it: a call's shape is fixed per call — give the list a bound (`list[T][0, 10]`) or pass it as a list", self.place_name(ctx, &place))).at(span).why_at(local.span, "declared here with no bound")),
                }
            }
            other => return err(Diagnostic::error(format!("`...` unpacks a list, and this is a `{}`", self.spell(other))).at(span)),
        };
        let elem = self.elem_type_of(&t, span, "unpacking")?;
        let elem = self.resolve(&elem);
        let elem_owned = elem.is_owned() && !matches!(elem, Ty::Bundle(_));
        let verb = match verb {
            Some(v) if !elem_owned => return err(Diagnostic::error(format!("`...{} {}`: its elements are plain `{}` values, which copy by themselves — write `...{}`", v.spelled(), self.place_name(ctx, &place), self.spell(&elem), self.place_name(ctx, &place))).at(span)),
            v => v,
        };
        Ok((count, verb))
    }

    fn known_length(&self, ctx: &FnCtx, place: &Place) -> Option<usize> {
        if !place.proj.is_empty() {
            return None;
        }
        ctx.literal_lengths.get(&place.base).copied()
    }

    /// Match one flattened argument to a parameter of `kind` and type `pty`.
    fn argument(&mut self, ctx: &mut FnCtx, fa: FlatArg, kind: ParamKind, pty: &Ty, fname: &str, pname: &str, variadic: bool) -> Result<Arg> {
        match fa {
            FlatArg::Spread { expr, verb, count, span } => {
                let place = self.place(ctx, &expr, PlaceUse::Read)?;
                let t = self.resolve(&place.ty);
                let elem = self.elem_type_of(&t, span, "unpacking")?;
                let elem = self.resolve(&elem);
                if !variadic {
                    return err(Diagnostic::error(format!("`...` unpacks into a variadic parameter, and `{}` has none: the values would land on a fixed parameter", fname)).at(span));
                }
                // Each element must fit the variadic's element type.
                if !self.unify(&elem, pty) && !matches!(elem, Ty::Bundle(_)) {
                    return err(Diagnostic::error(format!("`{}` takes `{}` values, and the elements of {} are `{}`", fname, self.spell(pty), self.place_name(ctx, &place), self.spell(&elem))).at(span));
                }
                let elem_owned = elem.is_owned() && !matches!(elem, Ty::Bundle(_));
                let inner = match kind {
                    ParamKind::Owned => {
                        if elem_owned {
                            match verb {
                                Some(Verb::Copy) => Arg::Copy(place),
                                Some(Verb::Give) => Arg::Give(place),
                                Some(v) => return err(Diagnostic::error(format!("`...{}` cannot hand elements to an owning parameter: say `...copy` (each duplicated) or `...give` (each moved out, the list left empty)", v.spelled())).at(span)),
                                None => return err(Diagnostic::error(format!("the elements of {} are owned `{}` values and `{}` takes ownership of each: say `...copy {}` or `...give {}`", self.place_name(ctx, &place), self.spell(&elem), fname, self.place_name(ctx, &place), self.place_name(ctx, &place))).at(span)),
                            }
                        } else {
                            Arg::Lend { place, mutable: false }
                        }
                    }
                    ParamKind::Lent | ParamKind::LentMut => {
                        if let Some(v) = verb {
                            return err(Diagnostic::error(format!("`{}` takes `lent ...{}`: `...` with no verb lends each element — drop the `{}`", fname, pname, v.spelled())).at(span));
                        }
                        Arg::Lend { place, mutable: kind == ParamKind::LentMut }
                    }
                };
                Ok(Arg::Spread { expr: Box::new(inner), count })
            }
            FlatArg::One(e) => self.one_argument(ctx, &e, kind, pty, fname, pname),
        }
    }

    /// A single written argument against a parameter.
    pub fn one_argument(&mut self, ctx: &mut FnCtx, e: &ast::Expr, kind: ParamKind, pty: &Ty, fname: &str, pname: &str) -> Result<Arg> {
        let what = if pname.starts_with("the ") { format!("`{}`'s {}", fname, pname) } else { format!("`{}`'s parameter '{}'", fname, pname) };
        match &e.kind {
            AK::Verb { verb, amp, expr } => {
                if *amp {
                    self.amp_count += 1;
                }
                let use_ = match verb {
                    Verb::Give => PlaceUse::Give,
                    Verb::Copy => PlaceUse::Copy,
                    Verb::Lend => PlaceUse::Lend,
                    Verb::LendMut => PlaceUse::LendMut,
                };
                // The verb must agree with the parameter.
                match (kind, verb) {
                    (ParamKind::Lent, Verb::Give) => return err(Diagnostic::error(format!("`{}` only looks at '{}' (it is `lent`); you tried to give it away — write `lend`", fname, pname)).at(e.span)),
                    (ParamKind::Lent, Verb::Copy) => return err(Diagnostic::error(format!("`{}` only looks at '{}' (it is `lent`): a copy is not needed — write `lend`", fname, pname)).at(e.span)),
                    (ParamKind::Lent, Verb::LendMut) => return err(Diagnostic::error(format!("`{}` only looks at '{}' (it is `lent`): `lend.mut` offers more than it takes — write `lend`", fname, pname)).at(e.span)),
                    (ParamKind::LentMut, Verb::Lend) => return err(Diagnostic::error(format!("`{}` may modify '{}' (it is `lent.mut`): write `lend.mut`", fname, pname)).at(e.span)),
                    (ParamKind::LentMut, Verb::Give | Verb::Copy) => return err(Diagnostic::error(format!("`{}` may modify '{}' (it is `lent.mut`): write `lend.mut`, not `{}`", fname, pname, verb.spelled())).at(e.span)),
                    (ParamKind::Owned, Verb::Lend | Verb::LendMut) => {
                        let whom = if pname.starts_with("the ") { pname.to_string() } else { format!("'{}'", pname) };
                        return err(Diagnostic::error(format!("`{}` takes ownership of {}, and you lent it: write `give` (it moves) or `copy` (a second one)", fname, whom)).at(e.span));
                    }
                    _ => {}
                }
                let place = self.place(ctx, expr, use_)?;
                let pt = self.resolve(&place.ty);
                let name = self.place_name(ctx, &place);
                let fb = ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty();
                self.check_verb_on_type(&pt, *verb, fb, e.span, &name)?;
                if !self.unify(&pt, pty) {
                    return err(Diagnostic::error(format!("{} is a `{}`, and {} is a `{}`", what, self.spell(pty), name, self.spell(&pt))).at(e.span));
                }
                Ok(match verb {
                    Verb::Give => Arg::Give(place),
                    Verb::Copy => Arg::Copy(place),
                    Verb::Lend => Arg::Lend { place, mutable: false },
                    Verb::LendMut => Arg::Lend { place, mutable: true },
                })
            }
            _ => {
                let v = self.expr(ctx, e)?;
                let vt = self.resolve(&v.ty);
                if let K::Read(place) = &v.kind {
                    if vt.is_owned() && !(ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty()) {
                        let name = self.place_name(ctx, place);
                        return match kind {
                            ParamKind::Owned => err(Diagnostic::error(format!("{} takes ownership: say what happens to {} — `give {}` (it moves) or `copy {}` (a second one)", what, name, name, name)).at(e.span)),
                            ParamKind::Lent => err(Diagnostic::error(format!("{} looks at its value: write `lend {}`", what, name)).at(e.span)),
                            ParamKind::LentMut => err(Diagnostic::error(format!("{} may modify its value: write `lend.mut {}`", what, name)).at(e.span)),
                        };
                    }
                }
                let v = self.expect_store(ctx, v, pty, &what)?;
                Ok(Arg::Value(v))
            }
        }
    }

    pub fn arg_type(&self, a: &Arg) -> Ty {
        match a {
            Arg::Value(e) => e.ty.clone(),
            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) => place.ty.clone(),
            Arg::Spread { expr, .. } => self.arg_type(expr),
        }
    }

    fn call_value(&mut self, ctx: &mut FnCtx, callee: &ast::Expr, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let c = self.expr(ctx, callee)?;
        let ct = self.resolve(&c.ty);
        let ft = match ct.base() {
            Ty::Fn(f) => (**f).clone(),
            Ty::Seq { .. } => return err(Diagnostic::error(format!("{} is a list, not a function: `'xs'[…]` calls, `'xs'[pos …]` indexes", self.expr_name(ctx, &c))).at(span)),
            other => return err(Diagnostic::error(format!("{} is a `{}`, not a function: `'f'[…]` calls a held function, and `[pos …]` indexes a list", self.expr_name(ctx, &c), self.spell(other))).at(span)),
        };
        let flat = self.flatten_args(ctx, args, span)?;
        if flat.len() != ft.params.len() {
            return err(Diagnostic::error(format!("this function takes {} argument{}, and {} {} given", ft.params.len(), if ft.params.len() == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).at(span));
        }
        let name = self.expr_name(ctx, &c);
        let mut hargs = Vec::new();
        for (i, (fa, (kind, pty))) in flat.into_iter().zip(&ft.params).enumerate() {
            let a = self.argument(ctx, fa, *kind, pty, &name, &format!("{}", i + 1), false)?;
            hargs.push(a);
        }
        let ow = self.call_otherwise(ctx, otherwise, &ft.fails, &ft.ret, used, span, &name)?;
        let callee = match c.kind {
            K::Local(id) => Callee::Value(Box::new(Expr { kind: K::Local(id), ty: c.ty.clone(), span: c.span })),
            _ => Callee::Value(Box::new(c)),
        };
        Ok(Expr { kind: K::Call { callee, args: hargs, otherwise: ow, fails: ft.fails.clone() }, ty: ft.ret, span })
    }

    fn extern_call(&mut self, ctx: &mut FnCtx, id: hir::ExternId, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let info = self.items.externs[id.0 as usize].clone();
        if info.is_let {
            return err(Diagnostic::error(format!("'{}' is C's variable, not a function", info.name)).at(span));
        }
        if ctx.tier != Some(ast::Tier::Nocheck) {
            return err(Diagnostic::error(format!("a call into C is a nocheck site: nothing about `{}` can be proven — write `nocheck` in front of the statement", info.name)).at(span).fix(None, format!("nocheck … {}[…] …;", info.name)));
        }
        let flat = self.flatten_args(ctx, args, span)?;
        if !info.variadic && flat.len() != info.params.len() {
            return err(Diagnostic::error(format!("`{}` takes {} argument{}, and {} {} given", info.name, info.params.len(), if info.params.len() == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).at(span));
        }
        if info.variadic && flat.len() < info.params.len() {
            return err(Diagnostic::error(format!("`{}` takes at least {} argument{}", info.name, info.params.len(), if info.params.len() == 1 { "" } else { "s" })).at(span));
        }
        let mut hargs = Vec::new();
        for (i, fa) in flat.into_iter().enumerate() {
            let FlatArg::One(e) = fa else {
                return err(Diagnostic::error("`...` cannot unpack into a C call").at(span));
            };
            if i < info.params.len() {
                let (pname, pty) = info.params[i].clone();
                hargs.push(self.c_argument(ctx, &e, &pty, &info.name, &pname)?);
            } else {
                // C's own varargs: each extra argument must be C-shaped.
                let v = self.expr(ctx, &e)?;
                let vt = self.resolve(&v.ty);
                if !vt.base().is_c_shaped(&self.items.structs) && !matches!(vt.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_))) {
                    return err(Diagnostic::error(format!("a `{}` cannot cross to C's varargs: only C-shaped values can", self.spell(&vt))).at(e.span));
                }
                hargs.push(Arg::Value(v));
            }
        }
        let fails: Vec<(String, Option<Ty>)> = if info.ranged { vec![("out_of_range".into(), None)] } else { Vec::new() };
        let ow = match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error(format!("an `extern fn` cannot fail; drop the `otherwise` after `{}`", info.name)).at(span));
                }
                Some(Box::new(self.otherwise(ctx, o, &fails, &info.ret, used, span)?))
            }
            None => {
                if !fails.is_empty() {
                    return err(Diagnostic::error(format!("`{}` can fail (out_of_range): its return is a range C promised, checked at the door — say what happens with `otherwise`", info.name)).at(span));
                }
                None
            }
        };
        Ok(Expr { kind: K::Call { callee: Callee::Extern(id), args: hargs, otherwise: ow, fails: fails.clone() }, ty: info.ret.clone(), span })
    }

    /// An argument to C: a value, or a lend that becomes the pointer.
    fn c_argument(&mut self, ctx: &mut FnCtx, e: &ast::Expr, pty: &Ty, fname: &str, pname: &str) -> Result<Arg> {
        if let Ty::Ptr { mutable, elem } = pty {
            match &e.kind {
                AK::Verb { verb: Verb::Lend, expr, amp } => {
                    if *amp {
                        self.amp_count += 1;
                    }
                    if *mutable {
                        return err(Diagnostic::error(format!("`{}`'s parameter '{}' is a `ptr.mut`: C may write through it, so write `lend.mut`", fname, pname)).at(e.span));
                    }
                    let place = self.place(ctx, expr, PlaceUse::Lend)?;
                    self.check_lend_as_ptr(ctx, &place, elem, e.span)?;
                    return Ok(Arg::Lend { place, mutable: false });
                }
                AK::Verb { verb: Verb::LendMut, expr, amp } => {
                    if *amp {
                        self.amp_count += 1;
                    }
                    let place = self.place(ctx, expr, PlaceUse::LendMut)?;
                    self.check_lend_as_ptr(ctx, &place, elem, e.span)?;
                    return Ok(Arg::Lend { place, mutable: true });
                }
                AK::Verb { verb, .. } => return err(Diagnostic::error(format!("`{}` hands C a pointer: `lend` (or `lend.mut`) becomes the address; `{}` does not", fname, verb.spelled())).at(e.span)),
                _ => {
                    // a held pointer, or a capture-free function for a callback slot
                    let v = self.expr(ctx, e)?;
                    let vt = self.resolve(&v.ty);
                    if !self.unify(&vt, pty) {
                        return err(Diagnostic::error(format!("`{}`'s parameter '{}' is a `{}`, and this is a `{}`", fname, pname, self.spell(pty), self.spell(&vt))).at(e.span));
                    }
                    return Ok(Arg::Value(v));
                }
            }
        }
        if let Ty::Fn(ft) = pty {
            let v = self.expr(ctx, e)?;
            let vt = self.resolve(&v.ty);
            if let K::FnLit { captures, .. } = &v.kind {
                if !captures.is_empty() {
                    return err(Diagnostic::error("a function handed to C cannot capture anything: C has nowhere to keep them — a C callback slot holds an address and nothing else".to_string()).at(e.span));
                }
            }
            if let Ty::Fn(got) = vt.base() {
                if got.params.len() != ft.params.len() {
                    return err(Diagnostic::error(format!("`{}`'s parameter '{}' wants a function taking {} argument(s); got a function taking {} argument(s)", fname, pname, ft.params.len(), got.params.len())).at(e.span));
                }
            }
            if !self.unify(&vt, pty) {
                return err(Diagnostic::error(format!("`{}`'s parameter '{}' is a `{}`, and this is a `{}`", fname, pname, self.spell(pty), self.spell(&vt))).at(e.span));
            }
            return Ok(Arg::Value(v));
        }
        self.one_argument(ctx, e, ParamKind::Owned, pty, fname, pname)
    }

    fn check_lend_as_ptr(&self, ctx: &FnCtx, place: &Place, elem: &Ty, span: Span) -> Result<()> {
        let pt = self.resolve(&place.ty);
        let ok = match (pt.base(), elem) {
            (Ty::Str(_), Ty::Int(IntW::Fixed { bits: 8, .. })) => true,
            (Ty::Seq { elem: e, .. }, want) => self.same(e, want) || (e.is_int() && want.is_int()),
            (Ty::Struct { .. }, Ty::Struct { .. }) => self.same(pt.base(), elem),
            (Ty::Int(_), Ty::Int(_)) | (Ty::Bin(_), Ty::Bin(_)) | (Ty::Bool, Ty::Bool) => true,
            _ => false,
        };
        if !ok {
            return err(Diagnostic::error(format!("a lend of {} (a `{}`) does not give C a `ptr[{}]`", self.place_name(ctx, place), self.spell(&pt), self.spell(elem))).at(span));
        }
        Ok(())
    }

    fn fn_literal(&mut self, ctx: &mut FnCtx, lit: &ast::FnLit, span: Span) -> Result<Expr> {
        let fn_id = self.declare_literal(lit, ctx.module, span)?;
        // Captures: resolved in the enclosing scope, each with its verb.
        let mut caps = Vec::new();
        let mut inst_caps = Vec::new();
        for c in &lit.captures {
            let Some(outer) = ctx.lookup(&c.name.name) else {
                return err(Diagnostic::error(format!("'{}' is not declared", c.name.name)).at(c.name.span));
            };
            if c.amp {
                self.amp_count += 1;
            }
            let ty = self.resolve(&ctx.locals[outer.0 as usize].ty);
            if lit.params.iter().any(|p| p.name.name == c.name.name) {
                return err(Diagnostic::error(format!("'{}' is both a capture and a parameter: they cannot share a name", c.name.name)).at(c.name.span));
            }
            inst_caps.push((c.name.name.clone(), c.verb, ty));
            caps.push((outer, c.verb));
        }
        let key = InstKey { fn_id, blanks: Vec::new(), bundle: None };
        let inst = self.instance(key);
        self.instances[inst.0 as usize].captures = inst_caps;
        // The literal's body sees the enclosing names only through `with`: check it now
        // with a note of what is outside, for the message.
        let outer_names: Vec<(String, Verb, Ty, hir::LocalId)> = ctx.locals.iter().enumerate().map(|(i, l)| (l.name.clone(), Verb::Lend, l.ty.clone(), hir::LocalId(i as u32))).collect();
        self.check_literal_now(inst, outer_names)?;
        let sig = self.instances[inst.0 as usize].sig.clone();
        let ty = Ty::Fn(Box::new(FnTy { params: sig.params, ret: sig.ret, fails: sig.fails }));
        let captures = caps.into_iter().map(|(outer, verb)| hir::CaptureBinding { verb, outer, inner: hir::LocalId(0) }).collect();
        Ok(Expr { kind: K::FnLit { inst, captures }, ty, span })
    }

    fn check_literal_now(&mut self, inst: hir::FnInst, outer: Vec<(String, Verb, Ty, hir::LocalId)>) -> Result<()> {
        // Pull it off the pending list and check it with the outer names known for messages.
        self.pending.retain(|i| *i != inst);
        let key = self.instances[inst.0 as usize].key.clone();
        let sig = self.instances[inst.0 as usize].sig.clone();
        let decl = self.items.fns[key.fn_id.0 as usize].clone();
        let body = decl.body.clone().unwrap();
        let mut ctx = self.new_ctx(inst, decl.module, "the function".to_string(), sig.ret.clone(), sig.fails.clone(), false);
        ctx.captured_outer = Some(outer);
        let captures = self.instances[inst.0 as usize].captures.clone();
        for (name, verb, ty) in &captures {
            let kind = match verb {
                Verb::Lend => LocalKind::Lent,
                Verb::LendMut => LocalKind::LentMut,
                _ => LocalKind::Owned,
            };
            ctx.declare(name, Local { name: name.clone(), ty: ty.clone(), mutability: None, kind, from_blank: false, span: decl.span });
        }
        let mut params = Vec::new();
        for (p, (kind, ty)) in decl.params.iter().zip(&sig.params) {
            let lk = match kind {
                ParamKind::Owned => LocalKind::Owned,
                ParamKind::Lent => LocalKind::Lent,
                ParamKind::LentMut => LocalKind::LentMut,
            };
            let id = ctx.declare(&p.name, Local { name: p.name.clone(), ty: ty.clone(), mutability: p.mutability, kind: lk, from_blank: false, span: p.span });
            params.push(id);
        }
        let block = self.block(&mut ctx, &body)?;
        if sig.ret != Ty::Nothing && !super::stmt::leaves_block(&block) {
            return err(Diagnostic::error(format!("this function returns a `{}` but can fall off the end: every path must `return` a value", self.spell(&sig.ret))).at(decl.span));
        }
        self.instances[inst.0 as usize].body = Some(hir::Body { locals: ctx.locals, params, ret: sig.ret, fails: sig.fails, block, span: decl.span, name: "an anonymous function".into(), is_main: false });
        Ok(())
    }

    // ------------------------------------------------------------ conversions

    fn conversion(&mut self, ctx: &mut FnCtx, target: &ast::Type, operand: &ast::Expr, otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let mut blanks = Vec::new();
        let t = self.items.resolve_type(target, ctx.module, &mut blanks, false)?;
        let v = self.expr(ctx, operand)?;
        let v = self.as_operand(ctx, v)?;
        let vt = self.resolve(&v.ty);
        let target_spelling = self.spell(&t);
        let mut fails: Vec<&str> = Vec::new();
        match (vt.base(), t.base()) {
            (Ty::Int(_), Ty::Int(_)) => {}
            (Ty::Int(_), Ty::Bin(_)) => {}
            (Ty::Bin(_), Ty::Int(_)) => fails.push("not_whole"),
            (Ty::Bin(_), Ty::Bin(_)) => {}
            (Ty::Int(_) | Ty::Bin(_) | Ty::Bool | Ty::Char(_), Ty::Str(_)) => {}
            (Ty::Str(_), Ty::Int(_) | Ty::Bin(_)) => fails.push("not_a_number"),
            (Ty::Str(_), Ty::Str(_)) => {}
            (Ty::Str(_), Ty::Char(_)) => fails.push("not_one_char"),
            (Ty::Char(from), Ty::Char(to)) => {
                let down = matches!((from, to), (Unit::Grapheme, Unit::Point) | (Unit::Grapheme, Unit::Byte) | (Unit::Point, Unit::Byte));
                if down {
                    fails.push(if *to == Unit::Point { "not_one_point" } else { "not_one_byte" });
                }
            }
            (Ty::Char(_), Ty::Int(_)) => return err(Diagnostic::error("a character does not convert to a number: `to.int32[`7`]` could mean 55 (its code) or 7 (its digit) — `char.code[…]` gives the code, `to.int32[to.str.utf8[…]] otherwise …` the digit").at(span)),
            (Ty::Int(_), Ty::Char(_)) => return err(Diagnostic::error("a number does not convert to a character: `char.point.from_code[…] otherwise …` builds one from a code").at(span)),
            (_, Ty::Bool) => return err(Diagnostic::error("nothing converts to `bool`: there is no truthiness — compare instead (`'n' !== 0`)").at(span)),
            (_, Ty::Seq { .. }) => return err(Diagnostic::error("there is no conversion to a list").at(span)),
            (_, Ty::Struct { .. }) | (_, Ty::Enum { .. }) => return err(Diagnostic::error(format!("there is no conversion to `{}`: a struct or enum value is built by name", target_spelling)).at(span)),
            (Ty::Duration, _) | (_, Ty::Duration) => return err(Diagnostic::error("a duration does not convert: compare or add durations directly").at(span)),
            _ => return err(Diagnostic::error(format!("there is no conversion from `{}` to `{}`", self.spell(&vt), target_spelling)).at(span)),
        }
        // Into a range type: fails out_of_range (never a check site).
        if let Some(b) = t.bound() {
            let _ = b;
            fails.push("out_of_range");
        }
        let fail_list: Vec<(String, Option<Ty>)> = fails.iter().map(|f| (f.to_string(), None)).collect();
        let ow = match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error(format!("`to.{}` cannot fail here; drop the `otherwise`", target_spelling)).at(span));
                }
                Some(Box::new(self.otherwise(ctx, o, &fail_list, &t, used, span)?))
            }
            None => {
                if !fails.is_empty() {
                    return err(Diagnostic::error(format!("`to.{}` can fail ({}): say what happens — `otherwise …`", target_spelling, fails.join(", "))).at(span));
                }
                None
            }
        };
        let fails_owned: Vec<String> = fails.iter().map(|s| s.to_string()).collect();
        Ok(Expr { kind: K::Conv { target: t.clone(), expr: Box::new(v), otherwise: ow, fails: fails_owned }, ty: t, span })
    }

    // ------------------------------------------------------------ builtins

    fn builtin_head(&self, path: &ast::Path, span: Span) -> Result<Option<BuiltinHead>> {
        let head = path.head();
        let rest: Vec<&str> = path.segs[1..].iter().map(|s| s.name.as_str()).collect();
        let b = match (head, rest.as_slice()) {
            ("list", ["length"]) => BuiltinHead::B(Builtin::ListLength),
            ("list", ["append"]) => BuiltinHead::B(Builtin::ListAppend),
            ("list", ["remove"]) => BuiltinHead::B(Builtin::ListRemove),
            ("list", ["slice"]) => BuiltinHead::B(Builtin::ListSlice),
            ("list", ["empty"]) => return err(Diagnostic::error("`list.empty` is a value, not a call: write it without brackets").at(span)),
            ("list", [other]) => return err(Diagnostic::error(format!("`list.{}` does not exist: a list has `length`, `append`, `remove`, `slice`", other)).at(span)),
            ("array", ["length"]) => BuiltinHead::B(Builtin::ArrayLength),
            ("array", ["slice"]) => BuiltinHead::B(Builtin::ArraySlice),
            ("array", [other]) => return err(Diagnostic::error(format!("`array.{}` does not exist: an array has `length` and `slice`", other)).at(span)),
            ("to", ["list"]) => BuiltinHead::B(Builtin::ToList),
            ("to", ["array"]) => BuiltinHead::B(Builtin::ToArray),
            ("map", ["length"]) => BuiltinHead::B(Builtin::MapLength),
            ("map", ["has"]) => BuiltinHead::B(Builtin::MapHas),
            ("map", ["get"]) => BuiltinHead::B(Builtin::MapGet),
            ("map", ["set"]) => BuiltinHead::B(Builtin::MapSet),
            ("map", ["remove"]) => BuiltinHead::B(Builtin::MapRemove),
            ("map", ["keys"]) => BuiltinHead::B(Builtin::MapKeys),
            ("map", ["values"]) => BuiltinHead::B(Builtin::MapValues),
            ("map", ["pairs"]) => BuiltinHead::B(Builtin::MapPairs),
            ("map", []) => BuiltinHead::MapLiteral,
            ("map", [other]) => return err(Diagnostic::error(format!("`map.{}` does not exist: a map has `length`, `has`, `get`, `set`, `remove`, `keys`, `values`, `pairs`", other)).at(span)),
            ("str", ["bytes"]) => BuiltinHead::B(Builtin::StrBytes),
            ("str", ["points"]) => BuiltinHead::B(Builtin::StrPoints),
            ("str", ["graphemes"]) => BuiltinHead::B(Builtin::StrGraphemes),
            ("str", ["length"]) => return err(Diagnostic::error("a text has no single length: `str.bytes`, `str.points` or `str.graphemes` — say which").at(span)),
            ("str", ["each"]) => return err(Diagnostic::error("say the unit after `str.each`: `str.each.grapheme[…]`, `.point`, or `.byte`").at(span)),
            ("str", ["each", unit]) => match *unit {
                "byte" => BuiltinHead::B(Builtin::StrEach(Unit::Byte)),
                "point" => BuiltinHead::B(Builtin::StrEach(Unit::Point)),
                "grapheme" => BuiltinHead::B(Builtin::StrEach(Unit::Grapheme)),
                other => return err(Diagnostic::error(format!("`str.each.{}` is not a unit: `byte`, `point` or `grapheme`", other)).at(span)),
            },
            ("str", [other, ..]) => return err(Diagnostic::error(format!("`str.{}` is std's, not the language's: the language owns only the units (`str.bytes`, `str.each.grapheme`, …), and `std::` is not available yet", other)).at(span)),
            ("char", ["code"]) => BuiltinHead::B(Builtin::CharCode),
            ("char", [unit, "from_code"]) => match *unit {
                "byte" => BuiltinHead::B(Builtin::CharFromCode(Unit::Byte)),
                "point" => BuiltinHead::B(Builtin::CharFromCode(Unit::Point)),
                "grapheme" => return err(Diagnostic::error("a grapheme has no single code: `char.point.from_code[…]` or `char.byte.from_code[…]`").at(span)),
                other => return err(Diagnostic::error(format!("`char.{}` is not a unit", other)).at(span)),
            },
            ("char", [..]) => return err(Diagnostic::error(format!("`{}` does not exist: `char.code[…]`, `char.point.from_code[…]`, `char.byte.from_code[…]`", path.spelled())).at(span)),
            ("ptr", ["read"]) => BuiltinHead::B(Builtin::PtrRead),
            ("ptr", ["write"]) => BuiltinHead::B(Builtin::PtrWrite),
            ("ptr", ["at"]) => BuiltinHead::B(Builtin::PtrAt),
            ("ptr", ["is_null"]) => BuiltinHead::B(Builtin::PtrIsNull),
            ("ptr", [other]) => return err(Diagnostic::error(format!("`ptr.{}` does not exist: a pointer has `read`, `write`, `at`, `is_null`", other)).at(span)),
            ("simd", ["fill"]) => BuiltinHead::B(Builtin::SimdFill),
            ("simd", ["load"]) => BuiltinHead::B(Builtin::SimdLoad),
            ("simd", ["store"]) => BuiltinHead::B(Builtin::SimdStore),
            ("simd", ["all"]) => BuiltinHead::B(Builtin::SimdAll),
            ("simd", ["any"]) => BuiltinHead::B(Builtin::SimdAny),
            ("simd", ["max"]) => BuiltinHead::B(Builtin::SimdMax),
            ("simd", ["min"]) => BuiltinHead::B(Builtin::SimdMin),
            ("simd", ["sum"]) => BuiltinHead::B(Builtin::SimdSum),
            ("simd", [other]) => return err(Diagnostic::error(format!("`simd.{}` does not exist: a vector has `fill`, `load`, `store`, `all`, `any`, `max`, `min`, `sum`", other)).at(span)),
            ("channel", ["send"]) => BuiltinHead::B(Builtin::ChannelSend),
            ("channel", ["take"]) => BuiltinHead::B(Builtin::ChannelTake),
            ("channel", [other]) => return err(Diagnostic::error(format!("`channel.{}` does not exist: a channel has `send` and `take`", other)).at(span)),
            _ => return Ok(None),
        };
        Ok(Some(b))
    }

    fn builtin_call(&mut self, ctx: &mut FnCtx, head: BuiltinHead, path: &ast::Path, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let b = match head {
            BuiltinHead::MapLiteral => {
                if otherwise.is_some() {
                    return err(Diagnostic::error("a map literal cannot fail; drop the `otherwise`").at(span));
                }
                return self.map_literal(ctx, args, span);
            }
            BuiltinHead::B(b) => b,
        };
        let name = path.spelled();
        let positional: Vec<&ast::Expr> = {
            let mut v = Vec::new();
            for a in args {
                match a {
                    AArg::Positional(e) => v.push(e),
                    _ => return err(Diagnostic::error(format!("`{}` takes values in order", name)).at(a.span())),
                }
            }
            v
        };
        let arity = |n: usize, this: &Self| -> Result<()> {
            if positional.len() != n {
                return err(Diagnostic::error(format!("`{}` takes {} argument{}, and {} {} given", name, n, if n == 1 { "" } else { "s" }, positional.len(), if positional.len() == 1 { "was" } else { "were" })).at(span));
            }
            let _ = this;
            Ok(())
        };
        let mut fails: Vec<(String, Option<Ty>)> = Vec::new();
        let (hargs, ty): (Vec<Arg>, Ty) = match &b {
            Builtin::ListLength | Builtin::ArrayLength | Builtin::MapLength | Builtin::StrBytes | Builtin::StrPoints | Builtin::StrGraphemes => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                match (&b, at.base()) {
                    (Builtin::ListLength, Ty::Seq { kind: SeqKind::Array, .. }) => return err(Diagnostic::error(format!("{} is an `array`: its length is `array.length[…]`", self.arg_name(ctx, &a))).at(span)),
                    (Builtin::ListLength, Ty::Seq { .. } | Ty::Bundle(_)) => {}
                    (Builtin::ArrayLength, Ty::Seq { kind: SeqKind::List, .. }) => return err(Diagnostic::error(format!("{} is a `list`: its length is `list.length[…]`", self.arg_name(ctx, &a))).at(span)),
                    (Builtin::ArrayLength, Ty::Seq { .. }) => {}
                    (Builtin::MapLength, Ty::Map { .. }) => {}
                    (Builtin::StrBytes | Builtin::StrPoints | Builtin::StrGraphemes, Ty::Str(_)) => {}
                    (Builtin::ListLength | Builtin::ArrayLength, Ty::Str(_)) => return err(Diagnostic::error("a text has no single length: `str.bytes`, `str.points` or `str.graphemes` — say which").at(span)),
                    _ => return err(Diagnostic::error(format!("`{}` takes a {}, and {} is a `{}`", name, builtin_subject(&b), self.arg_name(ctx, &a), self.spell(&at))).at(positional[0].span)),
                }
                let mut ty = Ty::u64();
                // A bounded list's length is bounded too.
                if let Some(Bound::Len { lo, hi, .. }) = at.bound() {
                    ty = Ty::Bounded { inner: Box::new(Ty::u64()), bound: Bound::Int { lo: *lo as i128, hi: *hi as i128 } };
                }
                (vec![a], ty)
            }
            Builtin::ListAppend => {
                arity(2, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, true)?;
                if let Arg::Lend { place, .. } = &a {
                    ctx.literal_lengths.remove(&place.base);
                }
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, kind } => {
                        if *kind == SeqKind::Array {
                            return err(Diagnostic::error(format!("{} is an `array`, which cannot grow or shrink: declare it a `list` to append", self.arg_name(ctx, &a))).at(span));
                        }
                        (**elem).clone()
                    }
                    other => return err(Diagnostic::error(format!("`list.append` takes a list, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let v = self.one_argument(ctx, positional[1], ParamKind::Owned, &elem, "list.append", "the element")?;
                if let Arg::Lend { .. } = v {
                    return err(Diagnostic::error("a list owns its elements: `append` takes a fresh value, a `copy`, or a `give` — never a lend").at(positional[1].span));
                }
                if !used {}
                (vec![a, v], Ty::Nothing)
            }
            Builtin::ListRemove => {
                arity(2, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, true)?;
                if let Arg::Lend { place, .. } = &a {
                    ctx.literal_lengths.remove(&place.base);
                }
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, kind } => {
                        if *kind == SeqKind::Array {
                            return err(Diagnostic::error(format!("{} is an `array`, which cannot grow or shrink", self.arg_name(ctx, &a))).at(span));
                        }
                        (**elem).clone()
                    }
                    other => return err(Diagnostic::error(format!("`list.remove` takes a list, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let i = self.expr(ctx, positional[1])?;
                if !self.resolve(&i.ty).is_int() {
                    return err(Diagnostic::error("`list.remove` takes a 1-based position").at(positional[1].span));
                }
                (vec![a, Arg::Value(i)], elem)
            }
            Builtin::ListSlice | Builtin::ArraySlice => {
                arity(3, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match (&b, at.base()) {
                    (Builtin::ListSlice, Ty::Seq { elem, kind: SeqKind::List }) | (Builtin::ArraySlice, Ty::Seq { elem, kind: SeqKind::Array }) => (**elem).clone(),
                    // A `:=` collection is neither yet, and slicing only reads.
                    (_, Ty::Seq { elem, kind: SeqKind::Free(_) }) => (**elem).clone(),
                    (Builtin::ListSlice, Ty::Seq { kind: SeqKind::Array, .. }) => return err(Diagnostic::error(format!("{} is an `array`: its slice is `array.slice[…]`", self.arg_name(ctx, &a))).at(span)),
                    (_, Ty::Seq { kind: SeqKind::List, .. }) => return err(Diagnostic::error(format!("{} is a `list`: its slice is `list.slice[…]`", self.arg_name(ctx, &a))).at(span)),
                    (_, other) => return err(Diagnostic::error(format!("`{}` takes a {}, and {} is a `{}`", name, builtin_subject(&b), self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let first = self.pos_arg(ctx, positional[1], &name)?;
                let last = self.pos_arg(ctx, positional[2], &name)?;
                // A slice owns its elements, and is a list whichever it came
                // from (`lists.md`); Precog may lower it to an array.
                (vec![a, first, last], Ty::Seq { elem: Box::new(elem), kind: SeqKind::List })
            }
            Builtin::ToList | Builtin::ToArray => {
                arity(1, self)?;
                let e = positional[0];
                // It takes the collection, so it says how (`lists.md`): a
                // bare name or a lend would copy without saying so.
                match &e.kind {
                    AK::Name(n) => return err(Diagnostic::error(format!("converting bare '{}' would copy it without saying so: `give '{}'` hands it over, `copy '{}'` keeps it", n.name, n.name, n.name)).at(e.span)),
                    AK::Verb { verb: Verb::Lend | Verb::LendMut, .. } => return err(Diagnostic::error("converting a lend would copy it without saying so: `copy` it or `give` it").at(e.span)),
                    AK::Path(p) if p.limb.is_none() && p.segs.len() == 1 && (crate::resolve::primitive(p.head()).is_some() || matches!(p.head(), "list" | "array" | "map")) => {
                        return err(Diagnostic::error(format!("`{}` keeps the element type, so it is not written: `{}[copy 'a']`", name, name)).at(e.span));
                    }
                    _ => {}
                }
                let elem = self.fresh();
                let any = Ty::Seq { elem: Box::new(elem), kind: SeqKind::Free(InferId(0)) };
                let a = self.one_argument(ctx, e, ParamKind::Owned, &any, &name, "the list or array")?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, .. } => (**elem).clone(),
                    other => return err(Diagnostic::error(format!("`to.list` / `to.array` take a list or an array, and this is a `{}`", self.spell(other))).at(e.span)),
                };
                let kind = if b == Builtin::ToList { SeqKind::List } else { SeqKind::Array };
                (vec![a], Ty::Seq { elem: Box::new(elem), kind })
            }
            Builtin::MapHas | Builtin::MapGet | Builtin::MapRemove => {
                arity(2, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, b == Builtin::MapRemove)?;
                let at = self.resolve(&self.arg_type(&a));
                let (kt, vt) = match at.base() {
                    Ty::Map { key, val, .. } => ((**key).clone(), (**val).clone()),
                    other => return err(Diagnostic::error(format!("`{}` takes a map, and {} is a `{}`", name, self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let k = self.key_arg(ctx, positional[1], &kt)?;
                let ty = match b {
                    Builtin::MapHas => Ty::Bool,
                    _ => {
                        fails.push(("missing".into(), None));
                        vt
                    }
                };
                (vec![a, k], ty)
            }
            Builtin::MapSet => {
                arity(3, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, true)?;
                let at = self.resolve(&self.arg_type(&a));
                let (kt, vt) = match at.base() {
                    Ty::Map { key, val, .. } => ((**key).clone(), (**val).clone()),
                    other => return err(Diagnostic::error(format!("`map.set` takes a map, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let k = self.one_argument(ctx, positional[1], ParamKind::Owned, &kt, "map.set", "the key")?;
                let v = self.one_argument(ctx, positional[2], ParamKind::Owned, &vt, "map.set", "the value")?;
                if matches!(k, Arg::Lend { .. }) || matches!(v, Arg::Lend { .. }) {
                    return err(Diagnostic::error("a map owns its keys and values: `set` takes a fresh value, a `copy`, or a `give` — never a lend").at(span));
                }
                (vec![a, k, v], Ty::Nothing)
            }
            Builtin::MapKeys | Builtin::MapValues | Builtin::MapPairs => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let (kt, vt, order) = match at.base() {
                    Ty::Map { key, val, order } => ((**key).clone(), (**val).clone(), *order),
                    other => return err(Diagnostic::error(format!("`{}` takes a map, and {} is a `{}`", name, self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                if order == MapOrder::Unordered {
                    return err(Diagnostic::error(format!("{} is a `map.unordered`, which promises never to be walked: `{}` needs a `map` or `map.ordered`", self.arg_name(ctx, &a), name)).at(span));
                }
                let elem = match b {
                    Builtin::MapKeys => kt,
                    Builtin::MapValues => vt,
                    _ => self.pair_struct(kt, vt),
                };
                (vec![a], Ty::Seq { elem: Box::new(elem), kind: SeqKind::Array })
            }
            Builtin::StrEach(unit) => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                if !at.is_text() {
                    return err(Diagnostic::error(format!("`{}` walks a text, and {} is a `{}`", name, self.arg_name(ctx, &a), self.spell(&at))).at(positional[0].span));
                }
                let elem = match unit {
                    Unit::Byte => Ty::int(false, 8),
                    u => Ty::Char(*u),
                };
                (vec![a], Ty::Seq { elem: Box::new(elem), kind: SeqKind::Array })
            }
            Builtin::CharCode => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                let v = self.as_operand(ctx, v)?;
                let vt = self.resolve(&v.ty);
                let ty = match vt.base() {
                    Ty::Char(Unit::Byte) => Ty::int(false, 8),
                    Ty::Char(Unit::Grapheme) => return err(Diagnostic::error("a grapheme has no single code: it may be several code points — walk it with `str.each.point` after `to.str.utf8[…]`").at(span)),
                    Ty::Char(_) => Ty::int(false, 32),
                    other => return err(Diagnostic::error(format!("`char.code` takes a character, and this is a `{}`", self.spell(other))).at(positional[0].span)),
                };
                (vec![Arg::Value(v)], ty)
            }
            Builtin::CharFromCode(unit) => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                let vt = self.resolve(&v.ty);
                if !vt.is_int() {
                    return err(Diagnostic::error(format!("`{}` takes a number (the code), and this is a `{}`", name, self.spell(&vt))).at(positional[0].span));
                }
                fails.push(("not_a_char".into(), None));
                (vec![Arg::Value(v)], Ty::Char(*unit))
            }
            Builtin::PtrRead | Builtin::PtrWrite | Builtin::PtrAt | Builtin::PtrIsNull => {
                if b != Builtin::PtrIsNull && ctx.tier != Some(ast::Tier::Nocheck) {
                    return err(Diagnostic::error(format!("`{}` is a nocheck site: nothing about C's memory can be proven — write `nocheck` in front of the statement", name)).at(span));
                }
                let n = match b {
                    Builtin::PtrRead | Builtin::PtrIsNull => 1,
                    _ => 2,
                };
                arity(n, self)?;
                let p = self.expr(ctx, positional[0])?;
                let pt = self.resolve(&p.ty);
                let (mutable, elem) = match pt.base() {
                    Ty::Ptr { mutable, elem } => (*mutable, (**elem).clone()),
                    other => return err(Diagnostic::error(format!("`{}` takes a pointer, and this is a `{}`", name, self.spell(other))).at(positional[0].span)),
                };
                match b {
                    Builtin::PtrRead => (vec![Arg::Value(p)], elem),
                    Builtin::PtrIsNull => (vec![Arg::Value(p)], Ty::Bool),
                    Builtin::PtrAt => {
                        let i = self.expr(ctx, positional[1])?;
                        if !self.resolve(&i.ty).is_int() {
                            return err(Diagnostic::error("`ptr.at` moves by a whole number of elements").at(positional[1].span));
                        }
                        (vec![Arg::Value(p), Arg::Value(i)], pt.base().clone())
                    }
                    _ => {
                        if !mutable {
                            return err(Diagnostic::error("`ptr.write` needs a `ptr.mut`: this pointer is read-only").at(span));
                        }
                        let v = self.expr(ctx, positional[1])?;
                        let v = self.expect_store(ctx, v, &elem, "what the pointer points at")?;
                        (vec![Arg::Value(p), Arg::Value(v)], Ty::Nothing)
                    }
                }
            }
            Builtin::ChannelSend => {
                arity(2, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Channel(e) => (**e).clone(),
                    other => return err(Diagnostic::error(format!("`channel.send` takes a channel, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let v = self.one_argument(ctx, positional[1], ParamKind::Owned, &elem, "channel.send", "the value")?;
                if let Arg::Lend { .. } = v {
                    return err(Diagnostic::error("a value sent on a channel is given, never lent: no two tasks may hold one thing").at(positional[1].span));
                }
                (vec![a, v], Ty::Nothing)
            }
            Builtin::ChannelTake => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Channel(e) => (**e).clone(),
                    other => return err(Diagnostic::error(format!("`channel.take` takes a channel, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                fails.push(("closed".into(), None));
                (vec![a], elem)
            }
            Builtin::SimdFill => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                // The lane type comes from where the vector is going.
                let lane = self.fresh();
                let v = self.expect_store(ctx, v, &lane, "every lane")?;
                let ty = Ty::Simd { elem: Box::new(lane), lanes: 0 };
                (vec![Arg::Value(v)], ty)
            }
            Builtin::SimdLoad => {
                arity(2, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, .. } => (**elem).clone(),
                    other => return err(Diagnostic::error(format!("`simd.load` fills a vector from a list or an array, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let idx = self.pos_arg(ctx, positional[1], &name)?;
                (vec![a, idx], Ty::Simd { elem: Box::new(elem), lanes: 0 })
            }
            Builtin::SimdStore => {
                arity(3, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, true)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, .. } => (**elem).clone(),
                    other => return err(Diagnostic::error(format!("`simd.store` writes into a list or an array, and {} is a `{}`", self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let idx = self.pos_arg(ctx, positional[1], &name)?;
                let v = self.expr(ctx, positional[2])?;
                let vt = self.resolve(&v.ty);
                match vt.base() {
                    Ty::Simd { elem: lane, .. } => {
                        if !self.unify(lane, &elem) {
                            return err(Diagnostic::error(format!("`simd.store` writes `{}` lanes into a list of `{}`", self.spell(lane), self.spell(&elem))).at(positional[2].span));
                        }
                    }
                    other => return err(Diagnostic::error(format!("`simd.store` writes a vector, and this is a `{}`", self.spell(other))).at(positional[2].span)),
                }
                (vec![a, idx, Arg::Value(v)], Ty::Nothing)
            }
            Builtin::SimdAll | Builtin::SimdAny => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                let v = self.as_operand(ctx, v)?;
                let vt = self.resolve(&v.ty);
                match vt.base() {
                    Ty::Simd { elem, .. } if elem.base() == &Ty::Bool => {}
                    Ty::Simd { elem, .. } => return err(Diagnostic::error(format!("`{}` folds a vector of answers, and this holds `{}` lanes: a comparison gives one (`'a' < 'b'`)", name, self.spell(elem))).at(positional[0].span)),
                    other => return err(Diagnostic::error(format!("`{}` folds a vector of answers, and this is a `{}`", name, self.spell(other))).at(positional[0].span)),
                }
                (vec![Arg::Value(v)], Ty::Bool)
            }
            Builtin::SimdMax | Builtin::SimdMin | Builtin::SimdSum => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                let v = self.as_operand(ctx, v)?;
                let vt = self.resolve(&v.ty);
                let lane = match vt.base() {
                    Ty::Simd { elem, .. } if elem.is_numeric() => (**elem).clone(),
                    Ty::Simd { elem, .. } => return err(Diagnostic::error(format!("`{}` folds a vector of numbers, and this holds `{}` lanes", name, self.spell(elem))).at(positional[0].span)),
                    other => return err(Diagnostic::error(format!("`{}` folds a vector of numbers, and this is a `{}`", name, self.spell(other))).at(positional[0].span)),
                };
                (vec![Arg::Value(v)], lane)
            }
            Builtin::Std(_) => unreachable!(),
        };
        let ow = self.call_otherwise(ctx, otherwise, &fails, &ty, used, span, &name)?;
        Ok(Expr { kind: K::Call { callee: Callee::Builtin(b), args: hargs, otherwise: ow, fails }, ty, span })
    }

    fn pair_struct(&mut self, kt: Ty, vt: Ty) -> Ty {
        // A synthetic struct `key`/`value` for map.pairs.
        let name = "pair";
        let existing = self.items.structs.iter().position(|s| s.name == name && s.fields.len() == 2 && self.same(&s.fields[0].ty, &kt) && self.same(&s.fields[1].ty, &vt));
        let id = match existing {
            Some(i) => StructId(i as u32),
            None => {
                let id = StructId(self.items.structs.len() as u32);
                self.items.structs.push(StructInfo { name: name.into(), path: name.into(), layout_c: false, fields: vec![FieldInfo { name: "key".into(), immut: true, ty: kt }, FieldInfo { name: "value".into(), immut: true, ty: vt }], blanks: Vec::new(), span: Span::new(crate::source::FileId(0), 0, 0) });
                id
            }
        };
        Ty::Struct { id, args: Vec::new() }
    }

    /// A builtin's collection argument: `lend 'xs'` (or `lend.mut` when it changes it).
    fn lent_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr, name: &str, mutates: bool) -> Result<Arg> {
        match &e.kind {
            AK::Verb { verb, amp, expr } => {
                if *amp {
                    self.amp_count += 1;
                }
                match (verb, mutates) {
                    (Verb::Lend, false) => {
                        let place = self.place(ctx, expr, PlaceUse::Lend)?;
                        Ok(Arg::Lend { place, mutable: false })
                    }
                    (Verb::LendMut, true) => {
                        let place = self.place(ctx, expr, PlaceUse::LendMut)?;
                        Ok(Arg::Lend { place, mutable: true })
                    }
                    (Verb::Lend, true) => err(Diagnostic::error(format!("`{}` changes the collection, so it takes `lend.mut`, not `lend`", name)).at(e.span)),
                    (Verb::LendMut, false) => {
                        let place = self.place(ctx, expr, PlaceUse::LendMut)?;
                        Ok(Arg::Lend { place, mutable: true })
                    }
                    (v, _) => err(Diagnostic::error(format!("`{}` looks at the collection: write `lend`{}, not `{}`", name, if mutates { ".mut" } else { "" }, v.spelled())).at(e.span)),
                }
            }
            AK::Name(_) | AK::Field { .. } | AK::Index { .. } => {
                let place = self.place(ctx, e, PlaceUse::Read)?;
                let pt = self.resolve(&place.ty);
                let pn = self.place_name(ctx, &place);
                if pt.is_owned() {
                    return err(Diagnostic::error(format!("`{}` takes a lend: write `lend{} {}`", name, if mutates { ".mut" } else { "" }, pn)).at(e.span));
                }
                Ok(Arg::Lend { place, mutable: mutates })
            }
            _ => {
                // a fresh value lives for the call
                let v = self.expr(ctx, e)?;
                if mutates {
                    return err(Diagnostic::error(format!("`{}` changes a collection you hold: `lend.mut` a variable", name)).at(e.span));
                }
                Ok(Arg::Value(v))
            }
        }
    }

    fn key_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr, kt: &Ty) -> Result<Arg> {
        // A key to look up is only looked at: a fresh value, or a lend, or a bare owned name (lent).
        let v = self.expr(ctx, e)?;
        let v = self.as_operand(ctx, v)?;
        let v = self.expect_store(ctx, v, kt, "the key")?;
        Ok(self.value_arg(v))
    }

    /// `pos i` where a position is wanted.
    fn pos_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr, name: &str) -> Result<Arg> {
        let AK::Pos(inner) = &e.kind else {
            return err(Diagnostic::error(format!("`{}` says the position with `pos`, as every index does: `{}[…, pos 'i']`", name, name)).at(e.span));
        };
        let v = self.expr(ctx, inner)?;
        let vt = self.resolve(&v.ty);
        if !vt.is_int() {
            return err(Diagnostic::error(format!("a position is a whole number, and this is a `{}`", self.spell(&vt))).at(e.span));
        }
        Ok(Arg::Value(v))
    }

    fn arg_name(&self, ctx: &FnCtx, a: &Arg) -> String {
        match a {
            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) => self.place_name(ctx, place),
            Arg::Value(e) => self.expr_name(ctx, e),
            Arg::Spread { expr, .. } => self.arg_name(ctx, expr),
        }
    }
}

pub enum BuiltinHead {
    B(Builtin),
    MapLiteral,
}

pub enum FlatArg {
    One(ast::Expr),
    Spread { expr: ast::Expr, verb: Option<Verb>, count: usize, span: Span },
}

fn builtin_subject(b: &Builtin) -> &'static str {
    match b {
        Builtin::ListLength | Builtin::ListAppend | Builtin::ListRemove | Builtin::ListSlice => "list",
        Builtin::ArrayLength | Builtin::ArraySlice => "array",
        Builtin::MapLength | Builtin::MapHas | Builtin::MapGet | Builtin::MapSet | Builtin::MapRemove | Builtin::MapKeys | Builtin::MapValues | Builtin::MapPairs => "map",
        Builtin::StrBytes | Builtin::StrPoints | Builtin::StrGraphemes | Builtin::StrEach(_) => "text",
        _ => "value",
    }
}

/// Every leaf a literal, every node arithmetic.
fn literal_only(e: &Expr) -> bool {
    match &e.kind {
        K::Int(_) => true,
        K::Unary { op: crate::ast::UnOp::Neg, expr } => literal_only(expr),
        // Every operator that gives a whole number of its operands' width:
        // arithmetic, shifts, bitwise, wrapping.
        K::Binary { op, lhs, rhs, .. } => {
            use crate::ast::BinOp as B;
            matches!(op, B::Add | B::Sub | B::Mul | B::Pow | B::Div | B::Mod | B::Shl | B::Shr | B::AndBit | B::OrBit | B::Xor | B::WrapAdd | B::WrapSub | B::WrapMul | B::WrapPow) && literal_only(lhs) && literal_only(rhs)
        }
        K::Unary { op: crate::ast::UnOp::NotBit, expr } => literal_only(expr),
        _ => false,
    }
}

/// Give every arithmetic node of a literal-only expression the place's type.
/// The leaves keep their values; a leaf the type cannot hold is then a
/// narrowing Precog refuses as certain.
fn retype_literal_arith(e: &mut Expr, t: &Ty) {
    match &mut e.kind {
        K::Binary { lhs, rhs, .. } => {
            retype_literal_arith(lhs, t);
            retype_literal_arith(rhs, t);
            e.ty = t.clone();
        }
        K::Unary { expr, .. } => {
            retype_literal_arith(expr, t);
            e.ty = t.clone();
        }
        _ => {}
    }
}

/// A whole number written out: `7`, or `-7`.
fn is_int_literal(e: &Expr) -> bool {
    match &e.kind {
        K::Int(_) => true,
        K::Unary { op: crate::ast::UnOp::Neg, expr } => matches!(expr.kind, K::Int(_)),
        _ => false,
    }
}

pub fn join_int(a: IntW, b: IntW) -> IntW {
    match (a, b) {
        (IntW::Free(_), _) | (_, IntW::Free(_)) => IntW::Free(InferId(0)),
        (IntW::Fixed { signed: s1, bits: b1 }, IntW::Fixed { signed: s2, bits: b2 }) => {
            if s1 == s2 {
                IntW::Fixed { signed: s1, bits: b1.max(b2) }
            } else {
                // signed meets unsigned: a signed type wide enough for both
                let bits = if b1 == b2 { (b1 * 2).min(128) } else { b1.max(b2) };
                IntW::Fixed { signed: true, bits }
            }
        }
    }
}

pub fn join_bin(a: BinW, b: BinW) -> BinW {
    match (a, b) {
        (BinW::Free(_), _) | (_, BinW::Free(_)) => BinW::Free(InferId(0)),
        (BinW::Fixed(x), BinW::Fixed(y)) => BinW::Fixed(x.max(y)),
    }
}

/// A type as it is written in a `to.…[…]`. A width Precog has not chosen yet
/// has no spelling, so the message names one and says the choice is the
/// writer's.
fn conv_spelling(spelled: &str) -> String {
    match spelled {
        "integer" => "int32".to_string(),
        "bin" => "bin64".to_string(),
        other => other.to_string(),
    }
}

fn article(s: &str) -> String {
    if s.starts_with(['a', 'e', 'i', 'o', 'u']) {
        format!("an `{}`", s)
    } else {
        format!("a `{}`", s)
    }
}

/// The narrowest unit that holds a character literal.
pub fn narrowest_unit(c: &str) -> Unit {
    let points = c.chars().count();
    if points == 1 {
        if c.len() == 1 {
            Unit::Byte
        } else {
            Unit::Point
        }
    } else {
        Unit::Grapheme
    }
}

fn same_literal(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (K::Int(x), K::Int(y)) => x == y,
        (K::Str(x), K::Str(y)) => x == y,
        (K::Char(x), K::Char(y)) => x == y,
        (K::Bool(x), K::Bool(y)) => x == y,
        (K::List(x), K::List(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| match (p, q) {
            (Arg::Value(p), Arg::Value(q)) => same_literal(p, q),
            _ => false,
        }),
        _ => false,
    }
}


impl Checker {
    /// Testing aid: `std::` typed as the oracle emulates it. Not std.
    fn std_stub(&mut self, ctx: &mut FnCtx, path: &ast::Path, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let name = path.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(".");
        let mut positional = Vec::new();
        for a in args {
            match a {
                AArg::Positional(e) => positional.push(e),
                _ => return err(Diagnostic::error("std takes values in order").at(a.span())),
            }
        }
        let mut hargs = Vec::new();
        let mut fails: Vec<(String, Option<Ty>)> = Vec::new();
        let ty = match name.as_str() {
            "print.stdout" | "print.stderr" | "printu.stdout" | "printu.stderr" | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_output" | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_error" | "printf.stdout" | "printf.stderr" => {
                for e in &positional {
                    let v = self.expr(ctx, e)?;
                    let v = self.as_operand(ctx, v)?;
                    hargs.push(Arg::Value(v));
                }
                Ty::Nothing
            }
            "exit" => {
                if positional.len() != 1 {
                    return err(Diagnostic::error("`std::exit` takes one value, the exit code").at(span));
                }
                let v = self.expr(ctx, positional[0])?;
                let v = self.expect_store(ctx, v, &Ty::int(false, 8), "the exit code")?;
                hargs.push(Arg::Value(v));
                Ty::Nothing
            }
            "input.line" | "input.all" => {
                if !positional.is_empty() {
                    return err(Diagnostic::error(format!("`std::{}` takes no arguments", name)).at(span));
                }
                fails.push(("closed".into(), None));
                Ty::utf8()
            }
            "time.wait" => {
                let v = self.expr(ctx, positional[0])?;
                hargs.push(Arg::Value(v));
                Ty::Nothing
            }
            "time.after" | "time.every" => {
                let v = self.expr(ctx, positional[0])?;
                hargs.push(Arg::Value(v));
                Ty::Channel(Box::new(Ty::Duration))
            }
            "div.trunc" | "mod.trunc" => {
                let a = self.expr(ctx, positional[0])?;
                let b = self.expr(ctx, positional[1])?;
                let t = self.resolve(&a.ty);
                hargs.push(Arg::Value(a));
                hargs.push(Arg::Value(b));
                t
            }
            "round" | "floor" | "ceil" | "trunc" => {
                let a = self.expr(ctx, positional[0])?;
                let a = self.as_operand(ctx, a)?;
                let t = self.resolve(&a.ty);
                hargs.push(Arg::Value(a));
                t
            }
            "c.errno" => Ty::int(true, 32),
            _ if name.starts_with("str.") => {
                for e in &positional {
                    let v = self.expr(ctx, e)?;
                    let v = self.as_operand(ctx, v)?;
                    hargs.push(Arg::Value(v));
                }
                match &name[4..] {
                    "split" => Ty::list(Ty::utf8()),
                    "join" | "trim" | "upper" | "lower" | "replace" => Ty::utf8(),
                    "contains" | "starts_with" | "ends_with" | "all_digits" => Ty::Bool,
                    "find.grapheme" | "find.point" | "find.byte" => {
                        fails.push(("missing".into(), None));
                        Ty::int(true, 32)
                    }
                    "at.grapheme" => {
                        fails.push(("out_of_range".into(), None));
                        Ty::Char(Unit::Grapheme)
                    }
                    "at.point" => {
                        fails.push(("out_of_range".into(), None));
                        Ty::Char(Unit::Point)
                    }
                    "at.byte" => {
                        fails.push(("out_of_range".into(), None));
                        Ty::Char(Unit::Byte)
                    }
                    "slice.grapheme" | "slice.point" | "slice.byte" => {
                        fails.push(("out_of_range".into(), None));
                        Ty::utf8()
                    }
                    other => return err(Diagnostic::error(format!("`std::str.{}` does not exist", other)).at(span)),
                }
            }
            _ => return err(Diagnostic::error(format!("`std::{}` does not exist", name)).at(span)),
        };
        let ow = self.call_otherwise(ctx, otherwise, &fails, &ty, used, span, &format!("std::{}", name))?;
        Ok(Expr { kind: K::Call { callee: Callee::Builtin(Builtin::Std(name)), args: hargs, otherwise: ow, fails }, ty, span })
    }
}
