//! Expressions, places, calls, verbs, conversions, `otherwise`.

use crate::ast::{self, Arg as AArg, BinOp, ExprKind as AK, Round, UnOp, Verb};
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
        // A call made for what it does: a borrow it hands back is dropped
        // at once, and owns nothing.
        let saved = ctx.lend_call_ok.take();
        if let AK::Call { .. } = &e.kind {
            ctx.lend_call_ok = Some(e.span);
        }
        let v = self.expr_in(ctx, e, true, false);
        ctx.lend_call_ok = saved;
        let v = v?;
        match &v.kind {
            K::Call { .. } | K::Conv { .. } => {}
            _ if matches!(e.kind, AK::Index { .. }) => {
                let AK::Index { base, .. } = &e.kind else { unreachable!() };
                let who = match &base.kind {
                    AK::Name(n) => format!("'{}'", n.name),
                    _ => "a list".into(),
                };
                return err(Diagnostic::error(format!("{}[…] on its own reads an item and drops it; to change it, `change {}[…] = …;`", who, who)).at(e.span));
            }
            _ => return err(Diagnostic::error("a value alone is not a statement: assign it, or call something with it").at(e.span)),
        }
        Ok(v)
    }

    fn expr_in(&mut self, ctx: &mut FnCtx, e: &ast::Expr, top: bool, used: bool) -> Result<Expr> {
        let span = e.span;
        match &e.kind {
            AK::Int(t) => {
                if let Ok(v) = t.parse::<i128>() {
                    return Ok(Expr { kind: K::Int(v), ty: FREE_INT, span });
                }
                // Past `i128`: kept as its digits, for a wide type to hold
                // (owner, 2026-10-03), and past the widest there is, refused.
                match crate::bigint::Big::parse(t) {
                    Some(b) if b.bits() <= crate::types::WIDEST_INT as u64 => Ok(Expr { kind: K::BigInt(b.to_string()), ty: FREE_INT, span }),
                    _ => err(Diagnostic::error(format!("`{}` is too large for any integer type: the widest there is holds {} bits", t, crate::types::WIDEST_INT)).at(span)),
                }
            }
            AK::Bin(t) => Ok(Expr { kind: K::Bin(t.clone()), ty: FREE_BIN, span }),
            // A deci whose width its place gives: `deci64` under `:=`.
            AK::Exact(t) => Ok(Expr { kind: K::Bin(t.clone()), ty: Ty::Deci(EXACT_FREE), span }),
            AK::Str(s) => Ok(Expr { kind: K::Str(s.clone()), ty: Ty::Str(Enc::Free(InferId(0))), span }),
            AK::Newline => Ok(Expr { kind: K::Str("\n".into()), ty: Ty::Str(Enc::Free(InferId(0))), span }),
            AK::Char(c) => {
                // Several code points can only be a grapheme (`types.md`: "`เกี่ยว`
                // can only be `grapheme`"); one leaves the unit to its use.
                let unit = match narrowest_unit(c) {
                    Unit::Grapheme => Unit::Grapheme,
                    _ => Unit::Free(InferId(0)),
                };
                Ok(Expr { kind: K::Char(c.clone()), ty: Ty::Char(unit), span })
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
                    return err(Diagnostic::error("a call that can fail must be the whole value: put it in its own declaration, then use the result").at(span));
                }
                self.call(ctx, path, args, otherwise.as_deref(), span, used)
            }
            AK::CallValue { callee, args, otherwise } => {
                if otherwise.is_some() && !top {
                    return err(Diagnostic::error("a call that can fail must be the whole value: put it in its own declaration, then use the result").at(span));
                }
                self.call_value(ctx, callee, args, otherwise.as_deref(), span, used)
            }
            // `'f'[2]` calls what `'f'` holds when it holds a function
            // (owner, 2026-09-27: `pos` is removed).
            AK::Index { base, index } if matches!(self.peek_ty(ctx, base).as_ref().map(|t| t.base()), Some(Ty::Fn(_))) => {
                self.call_value(ctx, base, &[AArg::Positional((**index).clone())], None, span, used)
            }
            AK::Field { .. } | AK::Index { .. } => {
                let place = self.place(ctx, e, PlaceUse::Read)?;
                let ty = place.ty.clone();
                Ok(Expr { kind: K::Read(place), ty, span })
            }
            AK::Verb { verb, expr, .. } => {
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
                    return err(Diagnostic::error("a conversion that can fail must be the whole value: put it in its own declaration, then use the result").at(span));
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
            AK::Spread { .. } => err(Diagnostic::error("`...` has no meaning here: it unpacks a list into a call's arguments — `sum[...'xs']`").at(span)),
            AK::SizeOf { align, ty } => {
                // The function's own generics are announced, so naming one
                // here is no typo — it just has no size or layout yet.
                let mut blanks: Vec<(String, Vec<crate::types::IfaceId>)> = ctx.blank_names.iter().map(|n| (n.clone(), Vec::new())).collect();
                let t = self.items.resolve_type(ty, ctx.module, &mut blanks, true)?;
                if t.contains_var() {
                    return err(Diagnostic::error("a blank has no size until it is filled: `size.of` takes a concrete type").at(ty.span));
                }
                Ok(Expr { kind: K::SizeOf { align: *align, ty: t }, ty: Ty::u64(), span })
            }
            AK::PtrCast { ty, value } => {
                // Nothing proves what the bytes are (owner, 2026-09-30).
                if !ctx.tier.is_some_and(ast::Tier::trusts) {
                    return err(Diagnostic::error("`ptr.cast` says the bytes are a different type, which the compiler cannot vouch for — it is a !check site: write `!check` in front of the statement").at(span));
                }
                // The function's own generics are announced, so naming one
                // here is no typo — it just has no size or layout yet.
                let mut blanks: Vec<(String, Vec<crate::types::IfaceId>)> = ctx.blank_names.iter().map(|n| (n.clone(), Vec::new())).collect();
                let t = self.items.resolve_type(ty, ctx.module, &mut blanks, true)?;
                if t.contains_var() {
                    return err(Diagnostic::error("`ptr.cast` takes a concrete type: a blank has no layout").at(ty.span));
                }
                if !t.is_c_shaped(&self.items.structs, &self.items.enums) {
                    let why = match t.base() {
                        Ty::Struct { id, .. } => format!("struct `{}` is not `layout.C`; its layout is Precog's, not C's", self.items.structs[id.0 as usize].name),
                        Ty::Enum { id, .. } => format!("enum `{}` is not `layout.C`; its layout is Precog's, not C's", self.items.enums[id.0 as usize].name),
                        other => format!("`{}` is not C-shaped", self.spell(other)),
                    };
                    return err(Diagnostic::error(format!("a pointer points at C-shaped data, so it casts only to a C-shaped type: {}", why)).at(ty.span));
                }
                let v = self.expr(ctx, value)?;
                let vt = self.resolve(&v.ty);
                // Whether it may write stays as it was (owner, 2026-09-30: a).
                let mutable = match vt.base() {
                    Ty::Ptr { mutable, .. } => *mutable,
                    other => return err(Diagnostic::error(format!("`ptr.cast` takes a pointer, and this is a `{}`: a pointer and a number never turn into each other", self.spell(other))).at(value.span)),
                };
                Ok(Expr { kind: K::PtrCast(Box::new(v)), ty: Ty::Ptr { mutable, elem: Box::new(t) }, span })
            }
            AK::Count(_) => err(Diagnostic::error("`count` labels `list.fill`'s count: `list.fill[0, count 10]`").at(span)),
            AK::Pos(_) => err(Diagnostic::error("`pos` says a position, and one only goes where a list is read, sliced or loaded: `'xs'[pos 2]`, `simd.load[lend 'xs', pos 'i']`").at(span)),
            AK::Match { value, arms } => self.match_value(ctx, value, arms, span),
            AK::OpValue { spelled, .. } => err(Diagnostic::error(format!("`op {}` alone does not say which `{}`: hand it where a function is wanted — a parameter, or `let 'f': fn[int32, int32] --> int32 = op {};`", spelled, spelled, spelled)).at(span)),
            AK::Range { .. } => err(Diagnostic::error("`range[…]` is only decided as what a `for` runs over; as a value on its own it is not").at(span)),
        }
    }

    /// What a place holds, looked at without using it: enough to tell a
    /// list read from a call of a held function.
    fn peek_ty(&self, ctx: &FnCtx, e: &ast::Expr) -> Option<Ty> {
        let t = match &e.kind {
            AK::Name(n) => ctx.locals[ctx.lookup(&n.name)?.0 as usize].ty.clone(),
            AK::Paren(inner) => self.peek_ty(ctx, inner)?,
            AK::Field { base, field } => match self.peek_ty(ctx, base)?.base() {
                Ty::Struct { id, args } => {
                    let info = &self.items.structs[id.0 as usize];
                    info.fields.iter().find(|f| f.name == field.name)?.ty.subst(args)
                }
                _ => return None,
            },
            AK::Index { base, .. } => match self.peek_ty(ctx, base)?.base() {
                Ty::Seq { elem, .. } => (**elem).clone(),
                Ty::Fn(f) => f.ret.clone(),
                _ => return None,
            },
            AK::CallValue { callee, .. } => match self.peek_ty(ctx, callee)?.base() {
                Ty::Fn(f) => f.ret.clone(),
                _ => return None,
            },
            _ => return None,
        };
        Some(self.resolve(&t))
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
        // A std name without its brackets: what std offers is called, and a
        // call needs `[…]` — the oracle's reason, before whether std is
        // there at all.
        if path.limb.as_ref().map_or(false, |l| l.name == "std") {
            return err(Diagnostic::error(format!("`{}` is called, and a call needs `[…]`: `{}[…]`", path.spelled(), path.spelled())).at(span));
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
                if decl.test {
                    return err(Diagnostic::error(format!("`{}` is a `test fn`: `cca test` runs it, and it is not a value to hold", decl.name)).at(span));
                }
                if !decl.blanks.is_empty() || decl.params.iter().any(|p| p.ty.is_none() || p.variadic) {
                    return err(Diagnostic::error(format!("`{}` has blanks, so it has no single type to hold: pass it where the blanks are filled, or write a version with the types", decl.name)).at(span));
                }
                let key = InstKey { fn_id: id, blanks: Vec::new(), bundle: None, owning: Vec::new() };
                let inst = self.instance(key);
                let sig = self.instances[inst.0 as usize].sig.clone();
                let ty = Ty::Fn(Box::new(FnTy { params: sig.params, ret: sig.ret, fails: sig.fails, owning: false }));
                Ok(Expr { kind: K::FnRef(inst), ty, span })
            }
            Ok(ItemRef::ExternFn(_)) => err(Diagnostic::error(format!("`{}` is an extern function: it is called (under `!check`), and is not a value to hold", path.spelled())).at(span)),
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
                    return err(Diagnostic::error(format!("`{}` is a `!mut` field of `{}` and cannot be written", field.name, info.name)).at(field.span));
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
                    // A lane is read, and written like a list item (owner,
                    // 2026-09-29: a: `change 'v'[3] = 30;`). Lending one out
                    // to be written through is not in the ruling.
                    Ty::Simd { .. } if use_ == PlaceUse::LendMut => {
                        return err(Diagnostic::error("one lane of a vector is read (`'v'[1]`) and written (`change 'v'[3] = 30;`); lending one out to be changed is not decided yet").at(e.span));
                    }
                    Ty::Simd { elem, .. } => (**elem).clone(),
                    Ty::Str(_) => return err(Diagnostic::error("a text has no `[…]`: it would not say which unit — the 3rd byte, point or grapheme? — use `std::str.at.<unit>` when std exists, or walk it with `str.each.<unit>`").at(e.span)),
                    Ty::Map { .. } => return err(Diagnostic::error("a map is looked up by key with `map.get[lend 'm', key] otherwise …`, not by position").at(e.span)),
                    Ty::Bundle(_) => return err(Diagnostic::error("a mixed bundle has no `[…]`: walk it with `for`").at(e.span)),
                    other => {
                        let who = match &base.kind {
                            AK::Name(n) => format!("'{}'", n.name),
                            _ => "this".into(),
                        };
                        return err(Diagnostic::error(format!("{} is a `{}`, neither a list nor a function: `'x'[…]` reads a list, or calls a function held in 'x'", who, self.spell(other))).at(e.span));
                    }
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

    /// An `extern let` is a place of its own kind: a read or write is a !check site.
    fn extern_place(&mut self, ctx: &mut FnCtx, id: hir::ExternId, use_: PlaceUse, span: Span) -> Result<Place> {
        let info = self.items.externs[id.0 as usize].clone();
        if !ctx.tier.is_some_and(ast::Tier::trusts) {
            let what = if use_ == PlaceUse::Write { "writing" } else { "reading" };
            return err(Diagnostic::error(format!("{} C's variable '{}' is a !check site: nothing about C's memory can be proven — write `!check` in front of the statement", what, info.name)).at(span));
        }
        if use_ == PlaceUse::Write && !info.mutable {
            return err(Diagnostic::error(format!("'{}' is read-only: `extern.C mut '{}': …` says C's variable may be written", info.name, info.name)).at(span));
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
                    return err(Diagnostic::error(format!("'{}' cannot be changed: it is `!mut`", name)).at(span).why_at(local.span, "declared `!mut` here"));
                }
                match local.kind {
                    LocalKind::Lent => return err(Diagnostic::error(format!("'{}' is lent (shared) and cannot be changed: it belongs to whoever lent it — `lent.mut` would allow it", name)).at(span).why_at(local.span, "lent here")),
                    LocalKind::LoopVar => return err(Diagnostic::error(format!("'{}' belongs to the loop and cannot be changed: the header alone says what it is on every pass", name)).at(span).why_at(local.span, "the loop's variable")),
                    _ => {}
                }
            }
            PlaceUse::LendMut => {
                if local.mutability == Some(ast::Mutability::Immut) {
                    return err(Diagnostic::error(format!("'{}' is `!mut`: it cannot be lent mutably", name)).at(span));
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
                Proj::Index(_) => s.push_str("[…]"),
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
        // Bare numbers in a `deci` place are decimals, exactly as written
        // (owner, 2026-09-29): `let 'a': deci64 = 0.1 + 0.2;` is 0.3.
        // Without ASP, Signatar's literals are bit patterns (owner,
        // 2026-10-06): one lands as the type's bits, read as signed for an
        // `intN`, and one wider than those bits is refused (owner: "a").
        // What a literal means anywhere else is not decided.
        if let (K::Int(i), true) = (&v.kind, self.items.raw_files.contains(&v.span.file) && matches!(vt.base(), Ty::Int(IntW::Free(_)))) {
            match &t {
                Ty::Int(IntW::Fixed { signed, bits }) if t.exact_limit().is_none() => {
                    let (signed, bits) = (*signed, *bits);
                    let room = 1i128.checked_shl(bits).filter(|_| bits < 127);
                    if room.is_some_and(|room| *i >= room) {
                        return err(Diagnostic::error(format!("`{}` does not fit the {} bits of {}: without ASP a literal is a bit pattern, 0 to {}", i, bits, self.spell(&t), room.unwrap() - 1)).at(v.span));
                    }
                    let value = match room {
                        Some(room) if signed && *i >= room / 2 => *i - room,
                        _ => *i,
                    };
                    return Ok(Expr { kind: K::Int(value), ty: t.clone(), span: v.span });
                }
                // Not settled yet: the literal waits, as anywhere.
                Ty::Infer(_) | Ty::Int(IntW::Free(_)) => {}
                _ => return err(Diagnostic::error(format!("what a literal means in {} without ASP is not decided", self.spell(&t))).at(v.span)),
            }
        }
        if matches!(vt.base(), Ty::Deci(EXACT_FREE)) && matches!(t.base(), Ty::Bin(_) | Ty::Int(_)) {
            return err(Diagnostic::error(format!("a `*` literal is exactly what is written, a deci, and {} is {}, which cannot hold it exactly: drop the `*` for a bin, or make it a `deci`", what, article(&self.spell(&t)))).at(v.span));
        }
        if let Ty::Deci(bits) = t.base() {
            if !matches!(vt.base(), Ty::Deci(b) if *b != EXACT_FREE) && deci_group(&v) {
                let mut v = v;
                into_deci(&mut v, *bits)?;
                return Ok(v);
            }
        }
        // Bare numbers in a `brain16` place are brain16s (owner, 2026-10-02).
        if t.is_brain() && !vt.is_brain() && bare_number_group(&v) {
            let mut v = v;
            into_brain(&mut v);
            return Ok(v);
        }
        if let (K::Int(i), Ty::Bin(_)) = (&v.kind, t.base()) {
            return Ok(Expr { kind: K::Bin(i.to_string()), ty: t.base().clone(), span: v.span });
        }
        if let (K::Unary { op: UnOp::Neg, expr }, Ty::Bin(_)) = (&v.kind, t.base()) {
            if let K::Int(i) = &expr.kind {
                return Ok(Expr { kind: K::Bin(format!("-{}", i)), ty: t.base().clone(), span: v.span });
            }
        }
        // A text written out is ASCII or it is not, before anything runs.
        if let (Ty::Str(Enc::Ascii), Ty::Str(Enc::Free(_))) = (t.base(), vt.base()) {
            fn first_wide(e: &Expr) -> Option<Span> {
                match &e.kind {
                    K::Str(s) if !s.is_ascii() => Some(e.span),
                    K::Binary { lhs, rhs, .. } => first_wide(lhs).or_else(|| first_wide(rhs)),
                    _ => None,
                }
            }
            if let Some(at) = first_wide(&v) {
                return err(Diagnostic::error("this text is not ASCII, so it cannot be str.ascii").at(at));
            }
        }
        // `list.fill` of a bare number takes the list's element type.
        if let (K::Call { callee: Callee::Builtin(Builtin::ListFill), .. }, Ty::Seq { elem, .. }) = (&v.kind, t.base()) {
            if let Ty::Seq { elem: have, .. } = vt.base() {
                if !matches!(have.base(), Ty::Struct { .. } | Ty::Enum { .. } | Ty::Seq { .. } | Ty::Map { .. }) && have.base() != elem.base() {
                    let mut v = v;
                    let span = v.span;
                    if let K::Call { args, .. } = &mut v.kind {
                        if let Some(Arg::Value(x)) = args.first_mut() {
                            let taken = std::mem::replace(x, Expr { kind: K::Nothing, ty: Ty::Nothing, span });
                            *x = self.expect_store(ctx, taken, elem, "each copy")?;
                        }
                    }
                    v.ty = t.base().clone();
                    return Ok(v);
                }
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
        // typed place it takes the place's: `'m': uint64 =
        // 9223372036854775807 + 1` is a `uint64` sum, 2^63, not an
        // `integer` one that overflows.
        let mut v = v;
        // A vector whose lanes were left to its context takes them from the
        // place it goes: a field, an element, a parameter.
        if let (Ty::Simd { lanes: 0, .. }, Ty::Simd { elem, lanes }) = (vt.base(), t.base()) {
            if *lanes > 0 {
                let (elem, lanes) = ((**elem).clone(), *lanes);
                self.settle_lanes(&mut v, &elem, lanes);
            }
        }
        // `Ty::Infer` too: a shift, `not.bit` or wrapping operator of bare
        // numbers waits for its width from here (owner, 2026-09-23), and so
        // does every literal group inside it.
        if let (Ty::Int(IntW::Free(_)) | Ty::Infer(_), Ty::Int(IntW::Fixed { .. })) = (vt.base(), t.base()) {
            if literal_only(&v) {
                // Bare numbers with only plain math between them are worked
                // out exactly and checked where they land (owner,
                // 2026-09-23): `let 'tiny': int8 = (100 + 100) - 150;` is
                // 50, and the 200 in between is fine. A shift, `not.bit` or
                // wrapping operator still takes its width from here.
                // In between, the limit is the width it lands in, at least
                // 64 bits (owner, 2026-10-03): `2 pow 255` into a
                // `!std_uint256`.
                let landing = match t.base() {
                    Ty::Int(IntW::Fixed { bits, .. }) if bits & crate::types::EXACT_FLAG == 0 => *bits,
                    _ => 64,
                };
                // Bare numbers alone are worked out exactly, with no limit in
                // between but the largest value they reach (check claude:
                // `(4294967294 pow 12) mod.floor 2147483649` is 16777216).
                let reach = literal_group_bits(&v).unwrap_or(0);
                let to = if plain_group(&v) { Ty::exact_at(landing.max(reach)) } else { t.base().clone() };
                retype_literal_arith(&mut v, &to);
            }
        }
        // A bare number takes its type from the place too, so beside a
        // typed operand it brings the place's width to the limit in between
        // (the oracle: `2 pow 'i'` into a `!std_uint256` is worked at 256
        // bits, `'i' times 'i'` at the 64 of its operands).
        if let (Some(limit), Ty::Int(IntW::Fixed { bits, .. })) = (vt.exact_limit(), t.base()) {
            if bits & crate::types::EXACT_FLAG == 0 && *bits > limit {
                raise_beside_literal(&mut v, *bits);
            }
        }
        // A list literal's items take the element type as their context, as
        // a declaration's value does (owner, 2026-09-29: a).
        // Items that took a bin or deci family change the list's type.
        let vt = if self.items_in_context(&mut v, t.base())? { self.resolve(&v.ty) } else { vt };
        // And a `match` used as a value hands the type to each case's value
        // (check claude, 2026-09-29: a case's bare numbers were worked out
        // at 64 bits, and past it the asserted build printed 16 for 25).
        if let K::Match { arms, .. } = &mut v.kind {
            let mut arms = std::mem::take(arms);
            for arm in arms.iter_mut() {
                if let Some(g) = arm.gives.take() {
                    arm.gives = Some(self.expect_store(ctx, g, &t, what)?);
                }
            }
            if let K::Match { arms: slot, .. } = &mut v.kind {
                *slot = arms;
            }
            v.ty = t.base().clone();
            return Ok(v);
        }
        // A pointer that may write is also one that reads: `ptr.mut[T]`
        // goes where `ptr[T]` is wanted, never the other way.
        if let (Ty::Ptr { mutable: true, elem: e1 }, Ty::Ptr { mutable: false, elem: e2 }) = (vt.base(), t.base()) {
            if self.unify(e1, e2) {
                let mut v = v;
                v.ty = t.base().clone();
                return Ok(v);
            }
        }
        if self.unify(&vt, &t) {
            return Ok(v);
        }
        // Say it in the writer's terms.
        let msg = match (vt.base(), t.base()) {
            (Ty::Brain, other) | (other, Ty::Brain) if other.is_numeric() => {
                let (have, want) = (self.spell(&vt), self.spell(&t));
                format!("{} is {} and this value is a {}, not {}: the brain float is a family of its own — convert it, `to.{}[…]`", what, article(&want), have, want, conv_spelling(&want))
            }
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
            // Two types spelled alike are two Limbs' (owner, 2026-09-30: one
            // Limb under two prefixes): each named with its own.
            _ if self.spell(&t) == self.spell(&vt) => format!("expected {} ({}), found {}", what.replace("the ", "a value of the "), self.spell_with_limb(&t), article(&self.spell_with_limb(&vt))),
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
                // A mask flips lane by lane (owner, 2026-09-27: a).
                if let Ty::Simd { elem, .. } = vt.base() {
                    if elem.base() == &Ty::Bool {
                        let ty = vt.base().clone();
                        return Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span });
                    }
                }
                self.expect_bool(&v, "`!`'s operand")?;
                Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty: Ty::Bool, span })
            }
            UnOp::NotBit => {
                // On an exact answer, at the size of its values: `not.bit
                // ('v' + 1)` with 'v' a `uint32` flips 32 bits
                // (`widths.cpls`), not 65.
                if vt.is_exact() {
                    let ty = self.size_of_values(&v).ok_or_else(|| Box::new(Diagnostic::error("not yet: `not.bit` on an exact answer whose operands have different sizes (or none left to see) — which size it flips is open").at(span)))?;
                    return Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span });
                }
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
            return err(Diagnostic::error(format!("`{}` is not defined for `{}`: declare `op {} ['a': {}, 'b': {}] --> …` to give it one", op.spelled(), self.spell(&lt), op.spelled(), self.spell(&lt), self.spell(&lt))).at(op_span));
        }
        let r = self.expr(ctx, rhs)?;
        let mut rt = self.resolve(&r.ty);
        let mut l = self.as_operand(ctx, l)?;
        let mut r = self.as_operand(ctx, r)?;
        // A comparison compares true values (owner, 2026-09-23), so a group
        // of bare numbers on either side is worked out exactly — not held at
        // `int64` beside another bare number (seed 439), nor squeezed into
        // the other side's width.
        let mut lt = lt;
        if op.is_comparison() && lt.is_int() && rt.is_int() {
            // At the other side's width, or the bare numbers' own size,
            // at least 64 bits (owner, 2026-10-03): `'m' == -2^126` beside
            // an `int128`.
            let side = |t: &Ty| match t.base() {
                Ty::Int(IntW::Fixed { bits, .. }) if bits & crate::types::EXACT_FLAG != 0 => bits & !crate::types::EXACT_FLAG,
                Ty::Int(IntW::Fixed { bits, .. }) => *bits,
                _ => 64,
            };
            let exact = Ty::exact_at(side(&lt).max(side(&rt)).max(literal_bits(&l)).max(literal_bits(&r)));
            if literal_only(&l) && matches!(lt.base(), Ty::Int(IntW::Free(_))) && matches!(l.kind, K::Binary { .. } | K::Unary { .. }) {
                retype_literal_arith(&mut l, &exact);
                lt = exact.clone();
            }
            if literal_only(&r) && matches!(rt.base(), Ty::Int(IntW::Free(_))) && matches!(r.kind, K::Binary { .. } | K::Unary { .. }) {
                retype_literal_arith(&mut r, &exact);
                rt = exact;
            }
        }
        // Every operator a lane's type has works on a vector of them, lane by
        // lane (`simd.md`, owner: a); a scalar stands for every lane.
        if lt.is_simd() || rt.is_simd() {
            return self.simd_binary(op, op_span, l, r, &lt, &rt, span);
        }
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::Mod(_) => {
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
                self.rounding_takes(op, op_span, &ty)?;
                if *op == BinOp::Pow && ty.is_int() {
                    if let K::Int(e) = &r.kind {
                        if *e < 0 {
                            return err(Diagnostic::error("a negative exponent on integers would produce a fraction: `xx` on integers takes a non-negative power").at(rhs.span));
                        }
                    }
                }
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
            }
            BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow | BinOp::DivWrap(_) => {
                // An exact answer wraps at the size of its values: working
                // at full width and keeping the low bits is the same
                // answer, so only the type has to say the size.
                let typed = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Fixed { .. }));
                // `xx~` wraps at the base's size alone: its exponent is a
                // count, like a shift's, not a value being wrapped (the
                // oracle's reading, `operators.md`; not ruled).
                if *op == BinOp::WrapPow && (typed(&lt) || lt.is_exact()) {
                    let ty = if lt.is_exact() { self.wrap_size(op, op_span, &[&l])? } else { lt.base().clone() };
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                }
                if lt.is_exact() || rt.is_exact() || (typed(&lt) && typed(&rt) && lt.base() != rt.base()) {
                    let ty = self.wrap_size(op, op_span, &[&l, &r])?;
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                }
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
                // Characters compare by content (`==`); an order between
                // them was never decided, and the oracle refuses one.
                if lt.is_char() {
                    return err(Diagnostic::error(format!("`{}` on characters has no meaning yet: `==` compares them, and `char.code[…]` gives the number under one", op.spelled())).at(op_span));
                }
                if !lt.is_numeric() && lt.base() != &Ty::Duration {
                    return err(Diagnostic::error(format!("`{}` compares numbers, and this is a `{}`", op.spelled(), self.spell(&lt))).at(op_span));
                }
                let r = self.expect_operand(ctx, r, &lt, op, op_span)?;
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::And | BinOp::Or | BinOp::AndThen | BinOp::OrElse => {
                self.expect_bool(&l, &format!("the left side of `{}`", op.spelled()))?;
                self.expect_bool(&r, &format!("the right side of `{}`", op.spelled()))?;
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr => {
                if !lt.is_int() || !rt.is_int() {
                    let bad = if !lt.is_int() { &lt } else { &rt };
                    return err(Diagnostic::error(format!("`{}` works on integers, and this is a `{}`", op.spelled(), self.spell(bad))).at(op_span));
                }
                // A shift works at the width of what it shifts: its count is
                // a count, exact or not. On an exact answer, the width of the
                // typed operands inside it, as `plus.wrap` takes it (owner,
                // 2026-09-23, confirmed 2026-10-03).
                if matches!(op, BinOp::Shl | BinOp::Shr) && (lt.is_exact() || rt.is_exact()) {
                    let ty = if lt.is_exact() {
                        self.size_of_values(&l).ok_or_else(|| self.open_size(op, op_span))?
                    } else if matches!(lt.base(), Ty::Int(IntW::Fixed { .. })) {
                        lt.base().clone()
                    } else if let Some(t) = self.size_of_values(&r) {
                        // A bare number shifted: the width of any operand
                        // whose type is known (owner, 2026-09-23: a), the
                        // count's typed operands here (check claude:
                        // `7 shr (('a' minus 1) mod.floor 32)`).
                        let mut l = l;
                        retype_literal_arith(&mut l, &t);
                        if let K::Int(_) = l.kind {
                            l.ty = t.clone();
                        }
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                    } else {
                        // Or from where the value goes.
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), op_span, op.spelled().to_string()));
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                    };
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                }
                if lt.is_exact() || rt.is_exact() {
                    let ty = match (lt.is_exact(), rt.is_exact()) {
                        (true, true) => {
                            let a = self.size_of_values(&l).ok_or_else(|| self.open_size(op, op_span))?;
                            let b = self.size_of_values(&r).ok_or_else(|| self.open_size(op, op_span))?;
                            if a != b {
                                return Err(self.open_size(op, op_span));
                            }
                            a
                        }
                        (true, false) if matches!(rt.base(), Ty::Int(IntW::Fixed { .. })) => rt.base().clone(),
                        (false, true) if matches!(lt.base(), Ty::Int(IntW::Fixed { .. })) => lt.base().clone(),
                        (true, false) => self.size_of_values(&l).ok_or_else(|| self.open_size(op, op_span))?,
                        _ => self.size_of_values(&r).ok_or_else(|| self.open_size(op, op_span))?,
                    };
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
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

    /// "Wrapping wraps at the size of its values" (`operators.md`): for an
    /// exact sum, the one type its typed operands share. `None` when they
    /// differ, or when there are none left to see (a `:=` name holding an
    /// exact answer) — which size then is open (asked 2026-09-23).
    fn size_of_values(&self, e: &Expr) -> Option<Ty> {
        match self.sizes_of_values(e)?.as_slice() {
            [one] => Some(one.clone()),
            _ => None,
        }
    }

    /// Every size among an exact answer's typed operands; `None` when one
    /// has no size to see.
    fn sizes_of_values(&self, e: &Expr) -> Option<Vec<Ty>> {
        let t = self.resolve(&e.ty);
        if !t.is_exact() {
            return match t.base() {
                Ty::Int(IntW::Fixed { .. }) => Some(vec![t.base().clone()]),
                _ => None,
            };
        }
        let mut sizes: Vec<Ty> = Vec::new();
        fn walk(this: &Checker, e: &Expr, sizes: &mut Vec<Ty>) -> bool {
            let t = this.resolve(&e.ty);
            if literal_only(e) {
                return true;
            }
            if !t.is_exact() {
                if let Ty::Int(IntW::Fixed { .. }) = t.base() {
                    sizes.push(t.base().clone());
                    return true;
                }
                return false;
            }
            match &e.kind {
                K::Binary { lhs, rhs, .. } => walk(this, lhs, sizes) && walk(this, rhs, sizes),
                // A negative keeps its operand's size (check claude:
                // `(-('a' minus 1)) shr 1` with an `int8` 'a').
                K::Unary { expr, .. } => walk(this, expr, sizes),
                _ => false,
            }
        }
        if !walk(self, e, &mut sizes) {
            return None;
        }
        let mut out: Vec<Ty> = Vec::new();
        for s in sizes {
            if !out.contains(&s) {
                out.push(s);
            }
        }
        Some(out)
    }

    /// Wrapping is at one size, always written (owner, 2026-09-23: b): a
    /// sum of `int8` and `int16` says which with `to.int16[…]` first.
    fn wrap_size(&self, op: &BinOp, op_span: Span, sides: &[&Expr]) -> Result<Ty> {
        let mut all: Vec<Ty> = Vec::new();
        for e in sides {
            let t = self.resolve(&e.ty);
            if literal_only(e) && !t.is_exact() {
                continue;
            }
            match self.sizes_of_values(e) {
                Some(v) => {
                    for s in v {
                        if !all.contains(&s) {
                            all.push(s);
                        }
                    }
                }
                None => return Err(self.open_size(op, op_span)),
            }
        }
        match all.as_slice() {
            [one] => Ok(one.clone()),
            [] => Err(self.open_size(op, op_span)),
            many => {
                let names: Vec<String> = many.iter().map(|t| self.spell(t)).collect();
                let last = names.last().cloned().unwrap_or_default();
                err(Diagnostic::error(format!("`{}` wraps at one size, and this mixes {}: say which with `to.{}[…]` first", op.spelled(), names.join(" and "), last)).at(op_span))
            }
        }
    }

    fn open_size(&self, op: &BinOp, op_span: Span) -> Box<Diagnostic> {
        Box::new(Diagnostic::error(format!("`{}` on an exact answer works at the width of its typed operands, and these have different sizes (or none left to see): say which first — `to.uint32[…]`", op.spelled())).at(op_span))
    }

    /// A lane-wise operator. The rule is the scalar rule, N times.
    #[allow(clippy::too_many_arguments)]
    fn simd_binary(&mut self, op: &BinOp, op_span: Span, mut l: Expr, mut r: Expr, lt: &Ty, rt: &Ty, span: Span) -> Result<Expr> {
        let (lane, lanes) = match (lt.base(), rt.base()) {
            (Ty::Simd { elem, lanes }, Ty::Simd { elem: e2, lanes: l2 }) => {
                if !self.unify(elem, e2) {
                    return err(Diagnostic::error(format!("`{}` works lane by lane, and these hold different lanes: `{}` and `{}`", op.spelled(), self.spell(elem), self.spell(e2))).at(op_span));
                }
                if lanes != l2 && *lanes != 0 && *l2 != 0 {
                    return err(Diagnostic::error(format!("`{}` works lane by lane, and these are different lengths: {} and {}", op.spelled(), lanes, l2)).at(op_span));
                }
                // The side that says its lanes says them for both: a
                // `simd.fill[10]` beside a `simd[int32, 4]` is four int32s.
                let (a, b) = (self.resolve(elem), self.resolve(e2));
                let free = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_)) | Ty::Var(_));
                let lane = if free(&a) && !free(&b) { b } else { a };
                let lanes = (*lanes).max(*l2);
                self.settle_lanes(&mut l, &lane, lanes);
                self.settle_lanes(&mut r, &lane, lanes);
                (lane, lanes)
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
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => lane_r.is_numeric(),
            BinOp::Div(_) | BinOp::Mod(_) => {
                self.rounding_takes(op, op_span, &lane_r)?;
                lane_r.is_numeric()
            }
            BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr => lane_r.is_int(),
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => lane_r.is_numeric() || lane_r.base() == &Ty::Bool,
            BinOp::And | BinOp::Or => lane_r.base() == &Ty::Bool,
            BinOp::AndThen | BinOp::OrElse => {
                let plain = if *op == BinOp::AndThen { "and" } else { "or" };
                return err(Diagnostic::error(format!("`{}` works on a single bool: it stops once the answer is known, and a mask has an answer in every lane — `{}` works lane by lane", op.spelled(), plain)).at(op_span));
            }
            _ => false,
        };
        if !ok {
            return err(Diagnostic::error(format!("`{}` is not an operator a lane of `{}` has, so a vector of them does not have it either", op.spelled(), self.spell(&lane_r))).at(op_span));
        }
        // A comparison gives one answer per lane, never a single `bool`.
        let ty = if op.is_comparison() { Ty::Simd { elem: Box::new(Ty::Bool), lanes } } else { Ty::Simd { elem: Box::new(lane), lanes } };
        Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
    }

    /// A list literal's items, in the element type's context: bare numbers
    /// with plain math between them are exact and checked where they land;
    /// a shift, `not.bit` or wrapping operator takes the element's width;
    /// a nested list passes its own element type down.
    fn items_in_context(&mut self, v: &mut Expr, t: &Ty) -> Result<bool> {
        let (K::List(items), Ty::Seq { elem, .. }) = (&mut v.kind, t) else { return Ok(false) };
        let elem = self.resolve(elem);
        let mut moved = false;
        for item in items.iter_mut() {
            let hir::Arg::Value(item) = item else { continue };
            match elem.base() {
                Ty::Seq { .. } => {
                    moved |= self.items_in_context(item, &elem)?;
                }
                Ty::Int(IntW::Fixed { .. }) => {
                    if matches!(self.resolve(&item.ty).base(), Ty::Int(IntW::Free(_)) | Ty::Infer(_)) && literal_only(item) && !matches!(item.kind, K::Int(_)) {
                        let to = if plain_group(item) { Ty::exact() } else { elem.base().clone() };
                        retype_literal_arith(item, &to);
                    }
                }
                // A bare whole number among bins is a bin, and bare numbers
                // among decis are decimals, as in a declaration of one
                // (`types.md`: a literal takes its context's family).
                Ty::Bin(_) => {
                    let lit = match &item.kind {
                        K::Int(i) => Some(i.to_string()),
                        K::Unary { op: UnOp::Neg, expr } => match &expr.kind {
                            K::Int(i) => Some(format!("-{}", i)),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(text) = lit {
                        *item = Expr { kind: K::Bin(text), ty: elem.base().clone(), span: item.span };
                        moved = true;
                    }
                }
                Ty::Brain => {
                    if bare_number_group(item) {
                        into_brain(item);
                        moved = true;
                    }
                }
                Ty::Deci(bits) => {
                    if !matches!(self.resolve(&item.ty).base(), Ty::Deci(_)) && deci_group(item) {
                        into_deci(item, *bits)?;
                        moved = true;
                    }
                }
                _ => {}
            }
        }
        // The list is then of the element type, when every item now is.
        if moved {
            let all = items.iter().all(|a| match a {
                hir::Arg::Value(x) => self.resolve(&x.ty).base() == elem.base(),
                _ => false,
            });
            if all {
                if let Ty::Seq { kind, .. } = self.resolve(&v.ty).base().clone() {
                    v.ty = Ty::Seq { elem: Box::new(elem.base().clone()), kind };
                }
            }
        }
        Ok(moved)
    }

    /// A vector whose lanes were left to its context (`simd.fill[…]`, and
    /// arithmetic on one) takes them from the side that says them.
    fn settle_lanes(&mut self, e: &mut Expr, lane: &Ty, lanes: u32) {
        let Ty::Simd { elem, lanes: have } = self.resolve(&e.ty).base().clone() else { return };
        let free = matches!(elem.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_)) | Ty::Var(_));
        if have != 0 && !free {
            return;
        }
        let el = if elem.base() == &Ty::Bool && lane.base() != &Ty::Bool { (*elem).clone() } else { lane.clone() };
        e.ty = Ty::Simd { elem: Box::new(el), lanes };
        if let K::Binary { op, lhs, rhs, .. } = &mut e.kind {
            if !op.is_comparison() {
                self.settle_lanes(lhs, lane, lanes);
                self.settle_lanes(rhs, lane, lanes);
            }
        }
    }

    /// An operand read. A bare *owned* name in an operator is a view of it
    /// — nothing is handed over by comparing two texts. A plain value is
    /// simply read: it owns nothing, and a verb on one is an error
    /// everywhere else in the language, so it would be strange here.
    fn as_operand(&mut self, _ctx: &mut FnCtx, e: Expr) -> Result<Expr> {
        let mut e = e;
        settle_exact(&mut e)?;
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
    /// Every division says how it rounds (owner, 2026-10-02): a bare `div`
    /// or `mod` is refused naming what it did on this family, and
    /// `mod.near` on a number with a fraction leaves nothing over.
    fn rounding_takes(&self, op: &BinOp, op_span: Span, ty: &Ty) -> Result<()> {
        let t = self.resolve(ty);
        let fraction = matches!(t.base(), Ty::Bin(_) | Ty::Brain | Ty::Deci(_));
        match op {
            BinOp::Div(Round::Bare) | BinOp::Mod(Round::Bare) => {
                let w = if op.is_div() { "div" } else { "mod" };
                let (meant, why) = match (op.is_div(), fraction) {
                    (true, false) => ("div.floor", "what `div` did on whole numbers"),
                    (true, true) => ("div.near", "ordinary division"),
                    (false, _) => ("mod.floor", "what `mod` did, the divisor's sign"),
                };
                err(Diagnostic::error(format!("say how `{w}` rounds: `{w}.floor`, `{w}.ceil`, `{w}.trunc` or `{w}.near` — `{meant}` is {why}")).at(op_span))
            }
            BinOp::Mod(Round::Near) if fraction => {
                let family = match t.base() {
                    Ty::Deci(_) => "decis",
                    Ty::Brain => "brain16",
                    _ => "bins",
                };
                err(Diagnostic::error(format!("`mod.near` on {family} leaves nothing over: `div.near` there is ordinary division — for what a whole quotient leaves, `mod.floor`, `mod.ceil` or `mod.trunc`")).at(op_span))
            }
            _ => Ok(()),
        }
    }

    fn numeric_pair(&mut self, ctx: &mut FnCtx, l: Expr, r: Expr, op: &BinOp, op_span: Span) -> Result<(Expr, Expr, Ty)> {
        let lt = self.resolve(&l.ty);
        let rt = self.resolve(&r.ty);
        if matches!(lt.base(), Ty::Deci(_)) || matches!(rt.base(), Ty::Deci(_)) {
            return self.deci_pair(l, r, &lt, &rt, op, op_span);
        }
        if lt.is_brain() || rt.is_brain() {
            return self.brain_pair(l, r, &lt, &rt, op, op_span);
        }
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
        // Plain math is exact, and checked where it lands (owner,
        // 2026-09-23, `operators.md`): `+ - x xx` give the true answer, and a
        // comparison compares true values. So a bare number beside a typed
        // one keeps its value rather than taking the other's width —
        // `(20000 + 20000) + 'small'` is 40005 whatever 'small' is — and the
        // answer is the exact type, checked when it is stored, passed or
        // returned. Only bare numbers alone stay without a type: they are
        // worked out at compile time.
        let plain = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::Mod(_)) || op.is_comparison();
        // Bare numbers compared with bare numbers are worked out exactly
        // too: `3876992840477302775 + 7782534087963747351 > 0` is true — the
        // sum is one some 64-bit type holds — not a sum held at `int64`
        // that stops (check claude, seed 439).
        if op.is_comparison() {
            if let (Ty::Int(IntW::Free(_)), Ty::Int(IntW::Free(_))) = (lt.base(), rt.base()) {
                if literal_only(&l) && literal_only(&r) {
                    let (mut l, mut r) = (l, r);
                    retype_literal_arith(&mut l, &Ty::exact());
                    retype_literal_arith(&mut r, &Ty::exact());
                    return Ok((l, r, Ty::exact()));
                }
            }
        }
        if plain {
            if let (Ty::Int(a), Ty::Int(b)) = (lt.base(), rt.base()) {
                if matches!(a, IntW::Fixed { .. }) || matches!(b, IntW::Fixed { .. }) {
                    let (mut l, mut r) = (l, r);
                    // In between, the limit is the widest operand's width,
                    // at least 64 bits (owner, 2026-10-03): two `int128`s
                    // add at 128 bits, two `!std_uint256`s at 256.
                    let width = |w: &IntW| match w {
                        IntW::Fixed { bits, .. } if bits & crate::types::EXACT_FLAG != 0 => bits & !crate::types::EXACT_FLAG,
                        IntW::Fixed { bits, .. } => *bits,
                        IntW::Free(_) => 64,
                    };
                    let exact = Ty::exact_at(width(a).max(width(b)));
                    if literal_only(&l) && matches!(a, IntW::Free(_)) {
                        retype_literal_arith(&mut l, &exact);
                    }
                    if literal_only(&r) && matches!(b, IntW::Free(_)) {
                        retype_literal_arith(&mut r, &exact);
                    }
                    return Ok((l, r, exact));
                }
            }
        }
        // Bare numbers alone are worked out exactly (dc83c05), a group of
        // them as `div`'s or `mod`'s other side too: `'p' mod (((221 x 2)
        // mod 5) + 1)` with 'p' a `uint8` divides by 3, where the group took
        // `uint8` and its `221 x 2` was refused (check claude, seed 4698).
        // The division is then exact as well, and lands where it is stored
        // — as `'p' + (…)` already does. A lone literal still takes the
        // other side's type.
        if matches!(op, BinOp::Div(_) | BinOp::Mod(_)) {
            let group = |e: &Expr, t: &Ty| matches!(t.base(), Ty::Int(IntW::Free(_))) && literal_only(e) && !is_int_literal(e);
            let fixed = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Fixed { .. }));
            if (group(&l, &lt) && fixed(&rt)) || (group(&r, &rt) && fixed(&lt)) {
                let (mut l, mut r) = (l, r);
                let exact = Ty::exact();
                if group(&l, &lt) {
                    retype_literal_arith(&mut l, &exact);
                }
                if group(&r, &rt) {
                    retype_literal_arith(&mut r, &exact);
                }
                return Ok((l, r, exact));
            }
        }
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

    /// Both sides of an operator where one is a `brain16` (owner,
    /// 2026-10-02): a family of its own, so bare numbers beside one become
    /// brain16s and nothing else mixes in without `to.`.
    fn brain_pair(&mut self, l: Expr, r: Expr, lt: &Ty, rt: &Ty, op: &BinOp, op_span: Span) -> Result<(Expr, Expr, Ty)> {
        let (mut l, mut r) = (l, r);
        if lt.is_brain() && !rt.is_brain() && bare_number_group(&r) {
            into_brain(&mut r);
        }
        if rt.is_brain() && !lt.is_brain() && bare_number_group(&l) {
            into_brain(&mut l);
        }
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::Mod(_) | BinOp::Pow => {}
            _ if op.is_comparison() => {}
            _ => return err(Diagnostic::error(format!("`{}` works on whole numbers, and a brain16 is not one", op.spelled())).at(op_span)),
        }
        let (lt, rt) = (self.resolve(&l.ty), self.resolve(&r.ty));
        if lt.is_brain() && rt.is_brain() {
            return Ok((l, r, Ty::Brain));
        }
        let other = if lt.is_brain() { &rt } else { &lt };
        err(Diagnostic::error(format!("`{}` between a brain16 and {}: the brain float is a family of its own — convert one side first, `to.brain16[…]` or `to.bin32[…]`", op.spelled(), article(&self.spell(other)))).at(op_span))
    }

    /// Both sides of an operator where one is a `deci` (owner, 2026-09-29):
    /// bare numbers beside one take its width, and nothing else mixes —
    /// not another family, not another `deci` width.
    fn deci_pair(&mut self, l: Expr, r: Expr, lt: &Ty, rt: &Ty, op: &BinOp, op_span: Span) -> Result<(Expr, Expr, Ty)> {
        let (mut l, mut r) = (l, r);
        // `xx` takes a whole exponent (owner, 2026-09-29): a whole-number
        // literal or variable — a count — and rounds like the rest. A
        // fractional one is a root, which is std's math.
        if *op == BinOp::Pow {
            if let (Ty::Deci(b), Ty::Int(_)) = (lt.base(), rt.base()) {
                if matches!(rt.base(), Ty::Int(IntW::Free(_))) && literal_only(&r) {
                    retype_literal_arith(&mut r, &Ty::Int(IntW::Fixed { signed: true, bits: 64 }));
                }
                return Ok((l, r, Ty::Deci(*b)));
            }
            if matches!(lt.base(), Ty::Deci(_)) {
                return err(Diagnostic::error(format!("a deci power's exponent is an integer or a whole literal, and this one is a `{}`: roots are std's math", self.spell(&rt))).at(r.span));
            }
            return err(Diagnostic::error("a deci power's exponent is an integer or a whole literal: the deci goes on the left of `xx`").at(op_span));
        }
        match (lt.base(), rt.base()) {
            (Ty::Deci(b), other) if *b != EXACT_FREE && !matches!(other, Ty::Deci(x) if *x != EXACT_FREE) && deci_group(&r) => into_deci(&mut r, *b)?,
            (other, Ty::Deci(b)) if *b != EXACT_FREE && !matches!(other, Ty::Deci(x) if *x != EXACT_FREE) && deci_group(&l) => into_deci(&mut l, *b)?,
            // Two exact literals with nothing else to say: both `deci64`.
            (Ty::Deci(EXACT_FREE), Ty::Deci(EXACT_FREE)) => {
                into_deci(&mut l, 64)?;
                into_deci(&mut r, 64)?;
            }
            // An exact literal beside a bare bin literal: the bin one is a
            // decimal too, as beside any deci.
            (Ty::Deci(EXACT_FREE), _) if deci_group(&r) => {
                into_deci(&mut l, 64)?;
                into_deci(&mut r, 64)?;
            }
            (_, Ty::Deci(EXACT_FREE)) if deci_group(&l) => {
                into_deci(&mut l, 64)?;
                into_deci(&mut r, 64)?;
            }
            _ => {}
        }
        let (lt, rt) = (self.resolve(&l.ty), self.resolve(&r.ty));
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::Mod(_) => {}
            _ if op.is_comparison() => {}
            _ => return err(Diagnostic::error(format!("`{}` works on whole numbers, and a deci is not one", op.spelled())).at(op_span)),
        }
        match (lt.base(), rt.base()) {
            (Ty::Deci(a), Ty::Deci(b)) if a == b => Ok((l, r, Ty::Deci(*a))),
            (Ty::Deci(a), Ty::Deci(b)) => {
                let wide = format!("deci{}", (*a).max(*b));
                err(Diagnostic::error(format!("`{}` between a deci{} and a deci{}: say which width, `to.{}[…]`", op.spelled(), a, b, wide)).at(op_span))
            }
            (Ty::Deci(b), _) | (_, Ty::Deci(b)) => err(Diagnostic::error(format!("`{}` between a deci and another number family: there is no implicit conversion between numeric families; `to.deci{}[…]`", op.spelled(), b)).at(op_span)),
            _ => unreachable!("deci_pair without a deci"),
        }
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
        // A path picks only that Limb's operator (owner, 2026-10-03).
        let (limb, name) = match spelling.split_once("::") {
            Some((limb, name)) => {
                let here = self.items.limb_root(ctx.module);
                let Some(root) = self.items.limb_deps.get(&here).and_then(|d| d.get(limb)).copied() else {
                    return err(Diagnostic::error(format!("no Limb with the prefix `{}`: a Limb this one uses is listed in its `CCA.cplr`, and its own `CCA.cplr` says `prefix = \"{}\"`", limb, limb)).at(op_span));
                };
                (Some((limb, root)), name)
            }
            None => (None, spelling),
        };
        let Some(fn_id) = self.find_op_in(name, lt.base(), limb.map(|l| l.1)) else {
            if let Some((limb, _)) = limb {
                return err(Diagnostic::error(format!("the Limb `{}` has no operator `{}` for a `{}`: a path picks only that Limb's operator", limb, name, self.spell(&lt))).at(op_span));
            }
            let hint = match spelling {
                "*" => "multiply is `times`",
                "^" => "the power is std's `pow`",
                "%" => "remainder is `mod.floor` (or `.ceil`, `.trunc`, `.near`)",
                "&" | "|" | "!" => "the boolean operators are `and`, `or`, `!`; the bitwise ones `and.bit`, `or.bit`, `not.bit`; a lend is `lend 'x'`",
                "&+" | "&-" | "+%" | "-%" | "*%" => "wrapping arithmetic is `plus.wrap`, `minus.wrap`, `times.wrap`, `pow.wrap`",
                _ => "an operator between two values is one the language owns (`plus`, `times`, `mod.floor`, …) or one declared with `op`",
            };
            return err(Diagnostic::error(format!("`{}` is not an operator here: {}", spelling, hint)).at(op_span));
        };
        // Bare, it may be more than one Limb's (owner, 2026-10-06): refused
        // where it is written, and this Limb's own does not win.
        if limb.is_none() {
            let here = self.items.limb_root(ctx.module);
            let mut roots: Vec<crate::resolve::ModuleId> = Vec::new();
            for id in self.ops_matching(name, lt.base(), None, false) {
                let r = self.items.limb_root(self.items.fns[id.0 as usize].module);
                if !roots.contains(&r) {
                    roots.push(r);
                }
            }
            if roots.len() > 1 {
                let mut named: Vec<String> = roots.iter().map(|r| match self.items.limb_deps.get(&here).and_then(|d| d.iter().find(|(_, x)| *x == r)) {
                    Some((p, _)) => format!("`{}::{}`", p, name),
                    None => format!("`my.{}` (this Limb's)", name),
                }).collect();
                named.sort();
                return err(Diagnostic::error(format!("`{}` could be any of {}: write the path", name, named.join(", "))).at(op_span));
            }
        }
        // Another Limb's word operator follows its mark (owner,
        // 2026-10-06): `bare.never` needs the path, `bare.only` refuses it.
        let decl = &self.items.fns[fn_id.0 as usize];
        let (here, there) = (self.items.limb_root(ctx.module), self.items.limb_root(decl.module));
        if here != there {
            match (limb, self.items.bare_marks.get(&decl.name_span)) {
                (None, Some(ast::Bare::Never)) => {
                    let prefix = self.items.limb_deps.get(&here).and_then(|d| d.iter().find(|(_, r)| **r == there)).map(|(p, _)| p.clone()).unwrap_or_default();
                    return err(Diagnostic::error(format!("`{n}` is `{p}`'s and `bare.never`: write `{p}::{n}`", n = name, p = prefix)).at(op_span));
                }
                (Some((prefix, _)), Some(ast::Bare::Only)) => {
                    return err(Diagnostic::error(format!("`{p}::{n}` is `bare.only`: `{p}` forbids its path, so write `{n}`", n = name, p = prefix)).at(op_span));
                }
                _ => {}
            }
        }
        self.user_op_call(ctx, fn_id, spelling, op_span, lhs, rhs, span)
    }

    /// A user `op` applied: operands lent by default, `give` when the op says so.
    fn user_op_call(&mut self, ctx: &mut FnCtx, fn_id: FnId, spelling: &str, op_span: Span, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        let decl = self.items.fns[fn_id.0 as usize].clone();
        if !decl.blanks.is_empty() {
            return self.generic_op_call(ctx, fn_id, spelling, lhs, rhs, span);
        }
        let key = InstKey { fn_id, blanks: Vec::new(), bundle: None, owning: Vec::new() };
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

    /// A generic `op` (owner, 2026-09-29): its blanks filled by the
    /// operands, as a generic `fn`'s are by its arguments — a typed operand
    /// first, so a bare number beside it takes its type — then held to the
    /// blank's interface.
    fn generic_op_call(&mut self, ctx: &mut FnCtx, fn_id: FnId, spelling: &str, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        let decl = self.items.fns[fn_id.0 as usize].clone();
        let named: Vec<Ty> = decl.blanks.iter().map(|_| self.fresh()).collect();
        let ptys: Vec<Ty> = decl.params.iter().map(|p| match &p.ty {
            Some(t) => t.subst(&named),
            None => Ty::Nothing,
        }).collect();
        let mut vals = Vec::new();
        for side in [lhs, rhs] {
            if let AK::Verb { verb, .. } = &side.kind {
                return err(Diagnostic::error(format!("an operand takes no `{}`: a generic operator's operands are lent", verb.spelled())).at(side.span));
            }
            vals.push(self.expr(ctx, side)?);
        }
        let free = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_)) | Ty::Infer(_));
        for (v, pt) in vals.iter().zip(&ptys) {
            let vt = self.resolve(&v.ty);
            if !free(&vt) && !self.unify(&vt, pt) {
                return err(Diagnostic::error(format!("one blank, one type: `{}` takes both operands as the same `<T>`, and this is a `{}` beside a `{}`", spelling, self.spell(&vt), self.spell(&self.resolve(pt)))).at(v.span));
            }
        }
        let mut args = Vec::new();
        for (v, pt) in vals.into_iter().zip(&ptys) {
            let vt = self.resolve(&v.ty);
            let v = self.expect_store(ctx, v, pt, "the operand")?;
            args.push(match v.kind {
                K::Read(place) if vt.is_owned() => Arg::Lend { place, mutable: false },
                _ => Arg::Value(v),
            });
        }
        let mut blank_tys = Vec::new();
        for (i, t) in named.iter().enumerate() {
            let r = self.resolve(t);
            if matches!(r, Ty::Infer(_)) {
                return err(Diagnostic::error(format!("the blank `<{}>` of `{}` is not filled: neither operand says what it is", decl.blanks[i].0, spelling)).at(span));
            }
            for iface in decl.blanks[i].1.clone() {
                if let Err(reason) = self.satisfies(&r, iface) {
                    let iname = self.items.ifaces[iface.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` asks for `<{}>: {}`", self.spell(&r), iname, reason, spelling, decl.blanks[i].0, iname)).at(span));
                }
            }
            blank_tys.push(r);
        }
        let inst = self.instance(InstKey { fn_id, blanks: blank_tys, bundle: None, owning: Vec::new() });
        let sig = self.instances[inst.0 as usize].sig.clone();
        let rhs_arg = args.pop().unwrap();
        let lhs_arg = args.pop().unwrap();
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
        let mut vals = Vec::new();
        for it in items {
            vals.push(self.expr(ctx, it)?);
        }
        // A bare whole number takes its family from the other items (owner,
        // 2026-09-30: a): `[1, 2.5]` is two bins, whichever comes first.
        let bare = |e: &Expr| match &e.kind {
            K::Int(_) | K::BigInt(_) => true,
            K::Unary { op: UnOp::Neg, expr } => matches!(expr.kind, K::Int(_) | K::BigInt(_)),
            _ => false,
        };
        if let Some(lead) = vals.iter().find(|v| !bare(v)) {
            let lt = self.resolve(&lead.ty);
            if matches!(lt.base(), Ty::Bin(_) | Ty::Deci(_)) && vals.iter().any(|v| bare(v)) {
                elem = Some(lt);
            }
        }
        for v in vals {
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

    /// A struct's or enum's filled blanks keep what it announced they
    /// accept (owner, 2026-10-02). A blank not filled yet is asked later,
    /// where it is.
    fn kept_bounds(&mut self, owner: &str, names: &[String], bounds: &[Vec<IfaceId>], args: &[Ty], span: Span) -> Result<()> {
        for (i, bs) in bounds.iter().enumerate() {
            let Some(t) = args.get(i) else { continue };
            let r = self.resolve(t);
            if matches!(r, Ty::Infer(_) | Ty::Var(_)) {
                continue;
            }
            for iface in bs {
                if let Err(reason) = self.satisfies(&r, *iface) {
                    let iname = self.items.ifaces[iface.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` needs `<{}>: {}`", self.spell(&r), iname, reason, owner, names.get(i).cloned().unwrap_or_default(), iname)).at(span));
                }
            }
        }
        Ok(())
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
        self.kept_bounds(&info.name, &info.blanks, &info.bounds, &args, span)?;
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
        self.kept_bounds(&info.name, &info.blanks, &info.bounds, &targs, span)?;
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
        // std marks everything `bare.optional` (owner, 2026-10-06): its
        // names may drop `std::`, and the built-ins it owns may carry it.
        if path.limb.as_ref().is_some_and(|l| l.name == "std") && std_builtin(path) {
            let plain = ast::Path { limb: None, segs: path.segs.clone(), span: path.span };
            return self.call(ctx, &plain, args, otherwise, span, used);
        }
        if path.limb.is_none() && std_name(path) {
            if let crate::resolve::Bareness::Std = self.items.resolve_bare(path, ctx.module, true)? {
                let with = ast::Path { limb: Some(ast::Ident { name: "std".into(), span: path.span }), segs: path.segs.clone(), span: path.span };
                return self.call(ctx, &with, args, otherwise, span, used);
            }
        }
        // The prints are one family (owner, dc7e2e4): the old names say the
        // new ones, with std or without it.
        if path.limb.as_ref().map(|l| l.name == "std").unwrap_or(false) {
            let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
            let renamed = match segs.first() {
                Some(&"printu") => Some("print.useful"),
                Some(&"printf") => Some("print.format"),
                _ => None,
            };
            if let Some(new) = renamed {
                let rest = segs[1..].join(".");
                let (old, new) = if rest.is_empty() { (segs[0].to_string(), new.to_string()) } else { (format!("{}.{}", segs[0], rest), format!("{}.{}", new, rest)) };
                return err(Diagnostic::error(format!("std::{} is spelled std::{} now", old, new)).at(span));
            }
        }
        // Every print says its form (owner, 2026-10-02): `.stored` or
        // `.useful`, after `.weird` when there is one.
        if path.limb.as_ref().map(|l| l.name == "std").unwrap_or(false) && path.segs.first().is_some_and(|s| s.name == "print") {
            let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
            match segs.as_slice() {
                ["print", stream @ ("stdout" | "stderr")] => {
                    return err(Diagnostic::error(format!("say which form a print shows: `std::print.stored.{}[…]` (what is stored) or `std::print.useful.{}[…]` (the useful value) (owner, 2026-10-02)", stream, stream)).at(span));
                }
                ["print", "weird", stream @ ("stdout" | "stderr")] => {
                    return err(Diagnostic::error(format!("say which form a print shows: `std::print.weird.stored.{} <- …` (what is stored) or `std::print.weird.useful.{} <- …` (the useful value) (owner, 2026-10-02)", stream, stream)).at(span));
                }
                ["print", f @ ("stored" | "useful"), "weird", rest @ ..] => {
                    return err(Diagnostic::error(format!("`.weird` comes first, the strongest modifier first: `std::print.weird.{}.{} <- …`", f, rest.join("."))).at(span));
                }
                _ => {}
            }
        }
        // Moved into the language (owner, 2026-10-02).
        if path.limb.as_ref().map(|l| l.name == "std").unwrap_or(false) {
            let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
            if let [w @ ("div" | "mod"), "trunc"] = segs.as_slice() {
                return err(Diagnostic::error(format!("`std::{w}.trunc` is an operator in the language now: `'a' {w}.trunc 'b'`")).at(span));
            }
        }
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
                if path.segs.len() == 1 && path.limb.is_none() && args.iter().any(|a| matches!(a, AArg::Named(..))) && e.sentence.starts_with("there is no") {
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
            // Nothing calls a `test fn` (owner, 2026-09-29).
            ItemRef::Fn(id) if self.items.fns[id.0 as usize].test => {
                err(Diagnostic::error(format!("`{}` is a `test fn`: `cca test` runs it, and nothing else calls it", path.spelled())).at(span))
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
                // A typed variadic holds values of its type, whatever each
                // argument's own width (a bare number's is none): `...'nums':
                // int32` is a list of int32, not of the exact type.
                let typed = decl.params.last().and_then(|p| p.ty.as_ref()).is_some();
                match &a {
                    Arg::Spread { expr, count } => {
                        let lt = self.resolve(&self.arg_type(expr));
                        let et = self.elem_type_of(&lt, span, "unpacking")?;
                        match self.resolve(&et) {
                            Ty::Bundle(ts) if !typed => bundle_tys.extend(ts),
                            t => {
                                let t = if typed { self.resolve(&pty) } else { t };
                                let n = match self.resolve(&lt) {
                                    Ty::Bundle(ts) => ts.len(),
                                    _ => *count,
                                };
                                for _ in 0..n {
                                    bundle_tys.push(t.clone());
                                }
                            }
                        }
                    }
                    _ if typed => bundle_tys.push(self.resolve(&pty)),
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
            for iface in decl.blanks[i].1.clone() {
                if let Err(reason) = self.satisfies(&r, iface) {
                    let iname = self.items.ifaces[iface.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` asks for `<{}>: {}`", self.spell(&r), iname, reason, decl.name, decl.blanks[i].0, iname)).at(span));
                }
            }
            blank_tys.push(r);
        }
        for t in &implicit {
            let r = self.resolve(t);
            blank_tys.push(r);
        }
        // A parameter handed a function that carries what it captured owns
        // it in this instance, and gives it back at the end.
        let owning: Vec<usize> = hargs.iter().enumerate().filter(|(i, a)| *i < fixed && matches!(self.resolve(&self.arg_type(a)).base(), Ty::Fn(f) if f.owning)).map(|(i, _)| i).collect();
        let key = InstKey { fn_id: id, blanks: blank_tys, bundle: if has_variadic { Some(bundle_tys) } else { None }, owning };
        let inst = self.instance(key);
        let sig = self.instances[inst.0 as usize].sig.clone();
        let _ = path;
        let ow = self.call_otherwise(ctx, otherwise, &sig.fails, &sig.ret, used, span, &decl.name)?;
        // A borrow handed back is held by a name, handed straight to a
        // parameter or a print, or handed back again by a function that
        // says so (owner, 2026-09-29) — anywhere else it would be a second
        // owner of what it borrows.
        if self.items.fns[id.0 as usize].ret_lent.is_some() && ctx.lend_call_ok != Some(span) {
            return err(Diagnostic::error(format!("`{}` hands back a borrow, which is held by a name (`let 'l' := {}[…];`) or handed straight to a parameter: here it would be a second owner of what it borrows", self.items.fns[id.0 as usize].name, self.items.fns[id.0 as usize].name)).at(span));
        }
        self.lends_apart(ctx, &hargs, span)?;
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
                // `must.fail` catches the reasons itself (owner, 2026-09-29).
                if !fails.is_empty() && !ctx.must_fail {
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
                    // An arm gives the call's value, as a `match` case does
                    // (owner, 2026-09-29), or leaves.
                    let (body, given) = self.arm_block(ctx, &arm.body)?;
                    ctx.pop_scope();
                    let given = match given {
                        Some(_) if !used => return err(Diagnostic::error(format!("the `{}` arm gives a value, but the call's value is not used", arm.name.name)).at(arm.body.span)),
                        Some(g) => Some(self.expect_store(ctx, g, ret, "the call's value")?),
                        None => None,
                    };
                    if used && given.is_none() && !super::stmt::leaves_block(&body) {
                        return err(Diagnostic::error(format!("the `{}` arm must give the call's value (`{{ 0 }}`) or leave (`return`, `fail`, `break`, `continue` or `std::exit`)", arm.name.name)).at(arm.body.span));
                    }
                    out.push(hir::FailArm { index: fi, bind, body, gives: given });
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
                // A `:=` list of bare numbers unpacked into a typed variadic
                // takes its width, as a lend does: its elements are copied
                // across as they are.
                if place.proj.is_empty() && !matches!(elem, Ty::Bundle(_)) {
                    let want = Ty::Seq { elem: Box::new(self.resolve(pty)), kind: SeqKind::List };
                    if let Some(adopted) = adopt_widths(&t, &want) {
                        ctx.locals[place.base.0 as usize].ty = adopted;
                    }
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
        // `op +` takes its type from the parameter it goes to.
        if let AK::OpValue { op, spelled, unary } = &e.kind {
            if let Ty::Fn(ft) = self.resolve(pty).base() {
                let ft = (**ft).clone();
                return Ok(Arg::Value(self.op_value(ctx, op, spelled, *unary, &ft, e.span)?));
            }
        }
        match &e.kind {
            AK::Verb { verb, expr, .. } => {
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
                // A name whose numbers' width was never written (`:=` of a
                // list of bare numbers) takes it from here: Precog picks the
                // type from use (`types.md`), and a lend hands over the
                // value where it is, so the callee's width is the one it has
                // to be. Left at the exact width, `total[lend 'xs']` on a
                // `list[int32]` read the wrong layout.
                if place.proj.is_empty() {
                    let pty_r = self.resolve(pty);
                    if let Some(adopted) = adopt_widths(&pt, &pty_r) {
                        ctx.locals[place.base.0 as usize].ty = adopted;
                    }
                }
                // "A lend hands over the value where it is, so the sizes
                // must match" (owner, 2026-09-24: a; `ownership-syntax.md`):
                // a `list[int64]` lent to a `list[int32]`. Bounds change no
                // layout; a width not written and a blank `<T>` are not
                // compared.
                if matches!(verb, Verb::Lend | Verb::LendMut) {
                    let have = self.resolve(&ctx.locals[place.base.0 as usize].ty);
                    let have = if place.proj.is_empty() { have } else { pt.clone() };
                    let want = self.resolve(pty);
                    if !sizes_match(&have, &want) {
                        return err(Diagnostic::error(format!("{} is a `{}`, and {} is a `{}`: a lend hands over the value where it is, so the sizes must match", what, self.spell(&want), name, self.spell(&have))).at(e.span));
                    }
                }
                Ok(match verb {
                    Verb::Give => Arg::Give(place),
                    Verb::Copy => Arg::Copy(place),
                    Verb::Lend => Arg::Lend { place, mutable: false },
                    Verb::LendMut => Arg::Lend { place, mutable: true },
                })
            }
            _ => {
                let saved = ctx.lend_call_ok.take();
                if let AK::Call { .. } = &e.kind {
                    ctx.lend_call_ok = Some(e.span);
                }
                let v = self.expr(ctx, e);
                ctx.lend_call_ok = saved;
                let v = v?;
                // A borrow a call hands back, used where it stands, goes to a
                // parameter as a `lend` (or `lend.mut`) written there would.
                if let (Some(mutable), K::Call { callee: Callee::Fn(inst), .. }) = (self.call_lends(&v), &v.kind) {
                    let callee = self.items.fns[self.instances[inst.0 as usize].key.fn_id.0 as usize].name.clone();
                    match (kind, mutable) {
                        (ParamKind::Owned, _) => return err(Diagnostic::error(format!("`{}` takes ownership of '{}', and `{}` hands back a borrow: hold it by a name and `copy` it, or pass what it borrows", fname, pname, callee)).at(e.span)),
                        (ParamKind::Lent, true) => return err(Diagnostic::error(format!("`{}` only looks at '{}' (it is `lent`): `{}` hands back a `lent.mut` borrow, which offers more than it takes", fname, pname, callee)).at(e.span)),
                        (ParamKind::LentMut, false) => return err(Diagnostic::error(format!("`{}` may modify '{}' (it is `lent.mut`): `{}` hands back a `lent` borrow, which only looks", fname, pname, callee)).at(e.span)),
                        _ => {}
                    }
                }
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
                // An argument takes its parameter's type as its context, as a
                // declaration's value does (owner, 2026-09-29: a) — but a
                // generic parameter has no width to give.
                if let Ty::Infer(_) = self.resolve(&v.ty) {
                    if self.resolve(pty).contains_var() {
                        let vt = self.resolve(&v.ty);
                        let op = self.widths_pending.iter().find(|(t, _, _)| self.resolve(t) == vt).map(|(_, _, w)| w.clone()).unwrap_or_else(|| "this".into());
                        return err(Diagnostic::error(format!("`{}` needs to know which width it works in, and a generic parameter gives none: give an operand a type (`to.uint8[0] minus.wrap 1`)", op)).at(e.span));
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
            Ty::Seq { .. } if self.items.strict_files.contains(&span.file) => return err(Diagnostic::error(format!("a list read says it is a position: `{}[pos …]` — a `#strict` file writes it out", self.expr_name(ctx, &c))).at(span)),
            Ty::Seq { .. } if otherwise.is_some() => return err(Diagnostic::error(format!("reading {}[…] cannot fail — a position past the end is a check site — so it takes no `otherwise`", self.expr_name(ctx, &c))).at(span)),
            Ty::Seq { .. } => return err(Diagnostic::error(format!("{} is a list, and a list is read one position at a time: `{}[2]`", self.expr_name(ctx, &c), self.expr_name(ctx, &c))).at(span)),
            other => return err(Diagnostic::error(format!("{} is a `{}`, not a function: `'f'[…]` calls a held function, and `'xs'[2]` reads a list", self.expr_name(ctx, &c), self.spell(other))).at(span)),
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
        self.lends_apart(ctx, &hargs, span)?;
        Ok(Expr { kind: K::Call { callee, args: hargs, otherwise: ow, fails: ft.fails.clone() }, ty: ft.ret, span })
    }

    fn extern_call(&mut self, ctx: &mut FnCtx, id: hir::ExternId, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let info = self.items.externs[id.0 as usize].clone();
        if info.is_let {
            return err(Diagnostic::error(format!("'{}' is C's variable, not a function", info.name)).at(span));
        }
        if !ctx.tier.is_some_and(ast::Tier::trusts) {
            return err(Diagnostic::error(format!("a call into C is a !check site: nothing about `{}` can be proven — write `!check` in front of the statement", info.name)).at(span).fix(None, format!("!check … {}[…] …;", info.name)));
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
                if !vt.base().is_c_shaped(&self.items.structs, &self.items.enums) && !matches!(vt.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_))) {
                    return err(Diagnostic::error(format!("a `{}` cannot cross to C's varargs: only C-shaped values can", self.spell(&vt))).at(e.span));
                }
                hargs.push(Arg::Value(v));
            }
        }
        // What C hands back is checked where it comes in (owner, 2026-09-27):
        // the return, and what C wrote through a `ptr.mut` into Cyborg's own
        // list lent to it (a pointer C already owns is C's memory, read
        // through `ptr.read`) — unless `NO_FUCKING_CHECKS` trusts C too.
        let writes_back = info.params.iter().zip(&hargs).any(|((_, t), a)| matches!(t.base(), Ty::Ptr { mutable: true, elem } if elem.checked_at_door(&self.items.structs, &self.items.enums)) && matches!(a, Arg::Lend { mutable: true, .. }));
        let fails: Vec<(String, Option<Ty>)> = if (info.ranged || writes_back) && ctx.tier != Some(ast::Tier::NoChecks) { vec![("out_of_range".into(), None)] } else { Vec::new() };
        if (info.ranged || writes_back) && ctx.tier == Some(ast::Tier::NoChecks) {
            self.items.trusted_door = true;
        }
        let ow = match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error(format!("an `extern fn` cannot fail; drop the `otherwise` after `{}`", info.name)).at(span));
                }
                Some(Box::new(self.otherwise(ctx, o, &fails, &info.ret, used, span)?))
            }
            None => {
                if !fails.is_empty() && !ctx.must_fail {
                    let what = if info.ret.bound().is_some() { "its return is a range C promised" } else { "what C hands back is checked at the door — a C enum's number must be one of its cases, a ranged field in its range" };
                    let what = if info.ret.bound().is_some() { format!("{}, checked at the door", what) } else { what.to_string() };
                    return err(Diagnostic::error(format!("`{}` can fail (out_of_range): {} — say what happens with `otherwise`", info.name, what)).at(span));
                }
                None
            }
        };
        self.lends_apart(ctx, &hargs, span)?;
        Ok(Expr { kind: K::Call { callee: Callee::Extern(id), args: hargs, otherwise: ow, fails: fails.clone() }, ty: info.ret.clone(), span })
    }

    /// A `:=` list lent to C's pointer takes the pointer's width: C reads
    /// its elements at that width.
    fn adopt_from_ptr(&mut self, ctx: &mut FnCtx, place: &Place, pty: &Ty) {
        if !place.proj.is_empty() {
            return;
        }
        let pt = self.resolve(&place.ty);
        let pty = self.resolve(pty);
        if let Some(adopted) = adopt_widths(&pt, &pty) {
            ctx.locals[place.base.0 as usize].ty = adopted;
        }
    }

    /// An argument to C: a value, or a lend that becomes the pointer.
    fn c_argument(&mut self, ctx: &mut FnCtx, e: &ast::Expr, pty: &Ty, fname: &str, pname: &str) -> Result<Arg> {
        if let Ty::Ptr { mutable, elem } = pty {
            match &e.kind {
                AK::Verb { verb: Verb::Lend, expr, .. } => {
                    if *mutable {
                        return err(Diagnostic::error(format!("`{}`'s parameter '{}' is a `ptr.mut`: C may write through it, so write `lend.mut`", fname, pname)).at(e.span));
                    }
                    let place = self.place(ctx, expr, PlaceUse::Lend)?;
                    self.check_lend_as_ptr(ctx, &place, elem, e.span)?;
                    self.adopt_from_ptr(ctx, &place, pty);
                    return Ok(Arg::Lend { place, mutable: false });
                }
                AK::Verb { verb: Verb::LendMut, expr, .. } => {
                    let place = self.place(ctx, expr, PlaceUse::LendMut)?;
                    self.check_lend_as_ptr(ctx, &place, elem, e.span)?;
                    self.adopt_from_ptr(ctx, &place, pty);
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
            let ty = self.resolve(&ctx.locals[outer.0 as usize].ty);
            if lit.params.iter().any(|p| p.name.name == c.name.name) {
                return err(Diagnostic::error(format!("'{}' is both a capture and a parameter: they cannot share a name", c.name.name)).at(c.name.span));
            }
            inst_caps.push((c.name.name.clone(), c.verb, ty));
            caps.push((outer, c.verb));
        }
        let key = InstKey { fn_id, blanks: Vec::new(), bundle: None, owning: Vec::new() };
        let inst = self.instance(key);
        self.instances[inst.0 as usize].captures = inst_caps;
        // The literal's body sees the enclosing names only through `with`: check it now
        // with a note of what is outside, for the message.
        let outer_names: Vec<(String, Verb, Ty, hir::LocalId)> = ctx.locals.iter().enumerate().map(|(i, l)| (l.name.clone(), Verb::Lend, l.ty.clone(), hir::LocalId(i as u32))).collect();
        self.check_literal_now(inst, outer_names)?;
        let sig = self.instances[inst.0 as usize].sig.clone();
        let owning = caps.iter().any(|(_, v)| matches!(v, Verb::Copy | Verb::Give));
        let ty = Ty::Fn(Box::new(FnTy { params: sig.params, ret: sig.ret, fails: sig.fails, owning }));
        let captures = caps.into_iter().map(|(outer, verb)| hir::CaptureBinding { verb, outer, inner: hir::LocalId(0) }).collect();
        Ok(Expr { kind: K::FnLit { inst, captures }, ty, span })
    }

    /// `op +` as a value of type `ft` (owner, 2026-09-29): a function of
    /// two operands, written out as `fn ['a': T, 'b': T] -> R { return 'a'
    /// + 'b'; }` with the types it is wanted at.
    pub fn op_value(&mut self, ctx: &mut FnCtx, op: &crate::ast::BinOp, spelled: &str, unary: Option<UnOp>, ft: &FnTy, span: Span) -> Result<Expr> {
        use crate::ast::BinOp as B;
        // A function's arguments are worked out before it runs, so these
        // could skip nothing (owner, 2026-09-29).
        if matches!(op, B::AndThen | B::OrElse) {
            let plain = if *op == B::AndThen { "and" } else { "or" };
            return err(Diagnostic::error(format!("`op {}` cannot skip anything: a function's arguments are worked out before it runs — write `op {}`", spelled, plain)).at(span));
        }
        let arity = if unary.is_some() { 1 } else { 2 };
        if ft.params.len() != arity {
            if *op == B::Sub && ft.params.len() == 1 {
                return err(Diagnostic::error("`op -` subtracts; negation as a value is `op neg`").at(span));
            }
            return err(Diagnostic::error(format!("`op {}` takes {} operand{}, and this wants a function of {}", spelled, arity, if arity == 1 { "" } else { "s" }, ft.params.len())).at(span));
        }
        if !ft.fails.is_empty() {
            return err(Diagnostic::error(format!("`op {}` cannot fail, and this wants a function that can", spelled)).at(span));
        }
        let names = ["a", "b"];
        let params: Vec<crate::resolve::ParamDecl> = ft.params.iter().zip(names).map(|((k, t), n)| crate::resolve::ParamDecl { kind: *k, group: None, amp: false, mutability: None, variadic: false, name: n.to_string(), ty: Some(t.clone()), from_blank: false, span }).collect();
        let name = |n: &str| ast::Expr { kind: AK::Name(crate::ast::QName { name: n.to_string(), span }), span };
        let value = match unary {
            Some(u) => ast::Expr { kind: AK::Unary { op: u, expr: Box::new(name("a")) }, span },
            None => ast::Expr { kind: AK::Binary { op: op.clone(), op_span: span, lhs: Box::new(name("a")), rhs: Box::new(name("b")) }, span },
        };
        let body = crate::ast::Block { stmts: vec![crate::ast::Stmt { kind: crate::ast::StmtKind::Return(Some(value)), span }], span };
        let fn_id = FnId(self.items.fns.len() as u32);
        self.items.fns.push(crate::resolve::FnDeclInfo { name: format!("op {}", spelled), path: String::new(), module: ctx.module, vis: crate::ast::Vis::Private, params, ret: ft.ret.clone(), ret_lent: None, fails: Vec::new(), blanks: Vec::new(), body: Some(body), op: None, is_literal: true, test: false, span, name_span: span });
        let key = InstKey { fn_id, blanks: Vec::new(), bundle: None, owning: Vec::new() };
        let inst = self.instance(key);
        self.check_literal_now(inst, Vec::new())?;
        let sig = self.instances[inst.0 as usize].sig.clone();
        let ty = Ty::Fn(Box::new(FnTy { params: sig.params, ret: sig.ret, fails: sig.fails, owning: false }));
        Ok(Expr { kind: K::FnLit { inst, captures: Vec::new() }, ty, span })
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
        self.instances[inst.0 as usize].body = Some(hir::Body { locals: ctx.locals, params, ret: sig.ret, fails: sig.fails, block, span: decl.span, name: "an anonymous function".into(), is_main: false, is_test: false, ret_lent: None });
        Ok(())
    }

    // ------------------------------------------------------------ conversions

    /// While something is lent to change, nothing else in the same call
    /// borrows it (owner, 2026-10-02: a): `two[lend.mut 'u', lend 'u']`,
    /// and a borrow a call hands back counts as a lend of what it borrows.
    /// A borrow inside an argument that ends before the call (`list.length[lend 'u']`)
    /// is not one.
    fn lends_apart(&self, ctx: &FnCtx, args: &[Arg], span: Span) -> Result<()> {
        let per: Vec<Vec<(hir::LocalId, bool, Span)>> = args.iter().map(|a| {
            let mut v = Vec::new();
            self.arg_borrows(ctx, a, &mut v);
            v
        }).collect();
        for (i, bs) in per.iter().enumerate() {
            for (root, m, at) in bs {
                if !*m {
                    continue;
                }
                let again = per.iter().enumerate().any(|(j, other)| j != i && other.iter().any(|(r, _, _)| r == root));
                if again {
                    let name = ctx.locals[root.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("'{}' is lent `lend.mut` and lent again in the same call: while something is lent to change, nothing else may borrow it", name)).at(span).why_at(*at, "lent to change here"));
                }
            }
        }
        Ok(())
    }

    /// What an argument still borrows when the call starts: a lend, or the
    /// lends of a call whose borrow it hands back.
    fn arg_borrows(&self, ctx: &FnCtx, a: &Arg, out: &mut Vec<(hir::LocalId, bool, Span)>) {
        match a {
            Arg::Lend { place, mutable } => {
                for root in ctx.borrow_src.get(&place.base).cloned().unwrap_or_else(|| vec![place.base]) {
                    out.push((root, *mutable, place.span));
                }
            }
            Arg::Value(e) => {
                if let (Some(m), K::Call { args, .. }) = (self.call_lends(e), &e.kind) {
                    let mut inner = Vec::new();
                    for x in args {
                        self.arg_borrows(ctx, x, &mut inner);
                    }
                    out.extend(inner.into_iter().map(|(r, _, s)| (r, m, s)));
                }
            }
            _ => {}
        }
    }

    /// A value's family in a message: "integer", "bool", "deci".
    fn family_word(&self, t: &Ty) -> String {
        match t.base() {
            Ty::Int(_) => "integer".into(),
            Ty::Bin(_) => "bin".into(),
            Ty::Brain => "brain16".into(),
            Ty::Deci(_) => "deci".into(),
            Ty::Bool => "bool".into(),
            Ty::Enum { id, .. } => format!("a `{}` case", self.items.enums[id.0 as usize].name),
            _ => format!("a `{}`", self.spell(t)),
        }
    }

    fn conversion(&mut self, ctx: &mut FnCtx, target: &ast::Type, operand: &ast::Expr, otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let mut blanks = Vec::new();
        // `to.useful.deci64[…]`: a bin's shortest decimal (owner,
        // 2026-09-29), where `to.deci64[…]` is its exact value; and
        // `to.useful.str.utf8[…]`, the useful text, where `to.str.utf8` is
        // the exact one. `.useful` follows the verb (owner, 2026-09-30).
        // `to.stored.` / `to.useful.` (owner, 2026-10-02): where a value has
        // two readings — a number, a `bool` or a case as text, a bin as a
        // deci — the conversion says which.
        let (form, stripped) = conv_form(target)?;
        let target = stripped.as_ref().unwrap_or(target);
        // `to.int8.bits[…]` (owner, 2026-10-06): ASP's raw form.
        if let ast::TypeKind::Path { path, args } = &target.kind {
            if path.segs.len() >= 2 && path.segs.last().is_some_and(|s| s.name == "bits") {
                let mut p = path.clone();
                p.segs.pop();
                let inner = ast::Type { kind: ast::TypeKind::Path { path: p, args: args.clone() }, span: target.span };
                return self.to_bits(ctx, &inner, operand, otherwise, span);
            }
        }
        let t = self.items.resolve_type(target, ctx.module, &mut blanks, false)?;
        if let Some(u) = form {
            if !matches!(t.base(), Ty::Str(_) | Ty::Deci(_)) {
                let f = if u { "useful" } else { "stored" };
                return err(Diagnostic::error(format!("`to.{}` makes text or a deci: `to.{}.str.utf8[…]`, `to.{}.deci64[…]`; `to.{}` has one form", f, f, f, self.spell(&t))).at(span));
            }
        }
        let mut useful = form == Some(true);
        let mut v = self.expr(ctx, operand)?;
        // Bare numbers alone are worked out exactly, whatever is between
        // them: `to.uint64[0 / ((2 / 10047273927188643194) + 1)]` divides by
        // 1. Held at `int64`, the big one read as negative (seed 371).
        if literal_only(&v) && matches!(self.resolve(&v.ty).base(), Ty::Int(IntW::Free(_))) && matches!(t.base(), Ty::Int(_)) {
            retype_literal_arith(&mut v, &Ty::exact());
        }
        let v = self.as_operand(ctx, v)?;
        let vt = self.resolve(&v.ty);
        let target_spelling = self.spell(&t);
        let mut fails: Vec<&str> = Vec::new();
        // Only a bin has a second value as a deci; an integer, a deci or
        // text has the one, whichever form is named.
        if matches!(t.base(), Ty::Deci(_)) && !matches!(vt.base(), Ty::Bin(_) | Ty::Brain) {
            useful = false;
        }
        match (vt.base(), t.base()) {
            // Into or out of a `deci` (owner, 2026-09-29): the list is
            // shared with the code generator, which finds each failure's
            // arm by its place in it.
            // A brain16 widens to a bin64 exactly, and a deci reaches a
            // brain16 through one: the bin's conversions, failures and all.
            (Ty::Brain, Ty::Deci(_)) => fails.extend(crate::deci::conv_fails(&Ty::Bin(BinW::Fixed(64)), &t, None).unwrap_or_default()),
            (Ty::Deci(_), Ty::Brain) => fails.extend(crate::deci::conv_fails(&vt, &Ty::Bin(BinW::Fixed(64)), None).unwrap_or_default()),
            (Ty::Int(_) | Ty::Bin(_) | Ty::Str(_) | Ty::Deci(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Int(_) | Ty::Bin(_) | Ty::Str(_)) => {
                let literal = match &v.kind {
                    K::Int(i) => Some(i.unsigned_abs().to_string().len() as u32),
                    K::BigInt(d) => Some(d.trim_start_matches('-').len() as u32),
                    _ => None,
                };
                fails.extend(crate::deci::conv_fails(&vt, &t, literal).unwrap_or_default());
            }
            (Ty::Int(_), Ty::Int(_)) => {}
            (Ty::Int(_), Ty::Bin(_)) => {}
            // The brain float (owner, 2026-10-02): into it rounds, from a
            // number or from text; out of it, a bin32 or bin64 is exact,
            // a bin16 a check site past its largest, an integer as from a bin.
            (Ty::Int(_) | Ty::Bin(_) | Ty::Brain, Ty::Brain) => {}
            (Ty::Str(_), Ty::Brain) => fails.extend(["not_a_number", "out_of_range"]),
            (Ty::Brain, Ty::Bin(_)) => {}
            (Ty::Brain, Ty::Int(_)) => fails.extend(["not_whole", "out_of_range"]),
            (Ty::Brain, Ty::Str(_)) => {}
            // Not whole, or whole and too big for the integer (owner,
            // 2026-09-30: a) — a failure, as text too big is, not a site.
            (Ty::Bin(_), Ty::Int(_)) => fails.extend(["not_whole", "out_of_range"]),
            (Ty::Bin(_), Ty::Bin(_)) => {}
            // Into ASCII, only an ASCII character is one (owner,
            // 2026-09-30: "`otherwise` exists"); a number, a `bool` or a
            // `char.byte` always is.
            (Ty::Char(u), Ty::Str(Enc::Ascii)) if *u != Unit::Byte => fails.push("not_ascii"),
            (Ty::Int(_) | Ty::Bin(_) | Ty::Bool | Ty::Char(_), Ty::Str(_)) => {}
            // An enum case as text: `Size.big` exactly, `big` usefully
            // (owner, 2026-09-30). One that carries data is open.
            // An enum with a case that carries data is not text, whichever
            // case a value holds (owner, 2026-09-30: "refuse", then "a" —
            // decided by the type).
            (Ty::Enum { id, .. }, Ty::Str(_)) => {
                let info = &self.items.enums[id.0 as usize];
                if let Some(c) = info.cases.iter().find(|c| c.payload.is_some()) {
                    return err(Diagnostic::error(format!("a `{}` is not text: its case `{}` carries data, so no `{}` converts, whichever case it holds; `match` it and build the text you want, or print it directly (owner, 2026-09-30)", info.name, c.name, info.name)).at(span));
                }
            }
            (Ty::Struct { id, .. }, Ty::Str(_)) => {
                let info = &self.items.structs[id.0 as usize];
                let name = match &operand.kind {
                    ast::ExprKind::Name(n) => format!("'{}'", n.name),
                    _ => "…".to_string(),
                };
                let field = info.fields.first().map(|f| format!(".{}", f.name)).unwrap_or_default();
                return err(Diagnostic::error(format!("a `{}` is not text: build the text you want from its fields, e.g. `to.str.utf8[{}{}] ++ …`, or print it directly (owner, 2026-09-30)", info.name, name, field)).at(span));
            }
            (Ty::Map { .. }, Ty::Str(_)) => return err(Diagnostic::error("a map is not text: build the text you want from its entries, or print it directly (owner, 2026-09-30)").at(span)),
            // Not a number, or a number the type cannot hold however many
            // digits it has (owner, 2026-09-24: c). Never a check site.
            (Ty::Str(_), Ty::Int(_) | Ty::Bin(_)) => fails.extend(["not_a_number", "out_of_range"]),
            (Ty::Str(from), Ty::Str(Enc::Ascii)) if *from != Enc::Ascii => fails.push("not_ascii"),
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
            // Bytes read as text (owner, 2026-09-30: "yea"): UTF-8 only.
            (Ty::Seq { elem, .. }, Ty::Str(enc)) if matches!(elem.base(), Ty::Int(IntW::Fixed { signed: false, bits: 8 })) => {
                if *enc != Enc::Utf8 {
                    return err(Diagnostic::error(format!("only `to.str.utf8` reads bytes as text (owner, 2026-09-30); `to.{}` from bytes is not decided — convert to utf8 first", target_spelling)).at(span));
                }
                fails.push("not_utf8");
            }
            (Ty::Seq { .. }, Ty::Str(_)) => return err(Diagnostic::error("a list is not text: only a list of `uint8` converts, read as UTF-8 bytes (`to.str.utf8['bytes'] otherwise …`); join the items with `++`, or print the list directly (owner, 2026-09-30)").at(span)),
            (_, Ty::Seq { .. }) => return err(Diagnostic::error("there is no conversion to a list").at(span)),
            // A number from C becomes a C enum only through a check (owner,
            // 2026-09-27): one no case has fails `out_of_range`.
            (Ty::Int(_), Ty::Enum { id, .. }) if self.items.enums[id.0 as usize].numbers.is_some() => fails.push("out_of_range"),
            (Ty::Int(_), Ty::Enum { .. }) => return err(Diagnostic::error(format!("only a `layout.C enum` converts from a number, and `{}` is a Cyborg enum: its value is built by name", target_spelling)).at(span)),
            // And back: a C enum's number never fails (owner, 2026-09-28);
            // a narrower target is a narrowing site, as ever.
            (Ty::Enum { id, .. }, Ty::Int(_)) if self.items.enums[id.0 as usize].numbers.is_some() => {}
            (Ty::Enum { .. }, Ty::Int(_)) => return err(Diagnostic::error(format!("only a `layout.C enum` has numbers: `{}` is a Cyborg enum, whose cases are names", self.spell(&vt))).at(span)),
            (_, Ty::Struct { .. }) | (_, Ty::Enum { .. }) => return err(Diagnostic::error(format!("there is no conversion to `{}`: a struct or enum value is built by name", target_spelling)).at(span)),
            (Ty::Duration, _) | (_, Ty::Duration) => return err(Diagnostic::error("a duration does not convert: compare or add durations directly").at(span)),
            _ => return err(Diagnostic::error(format!("there is no conversion from `{}` to `{}`", self.spell(&vt), target_spelling)).at(span)),
        }
        // Into a range type: fails out_of_range (never a check site).
        if t.bound().is_some() && !fails.contains(&"out_of_range") {
            fails.push("out_of_range");
        }
        if useful && !fails.is_empty() && matches!(t.base(), Ty::Str(_)) && matches!(vt.base(), Ty::Bin(_) | Ty::Deci(_) | Ty::Enum { .. }) {
            return err(Diagnostic::error(format!("not yet: `to.useful.{}` of a `{}` into text that can fail has no shape in the generated code yet", target_spelling, self.spell(&vt))).at(span));
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
                if !fails.is_empty() && !ctx.must_fail {
                    return err(Diagnostic::error(format!("`to.{}` can fail ({}): say what happens — `otherwise …`", target_spelling, fails.join(", "))).at(span));
                }
                None
            }
        };
        // Asked last, as the oracle does: what cannot convert at all, or
        // cannot fail, is said first.
        if form.is_none() {
            let two_texts = matches!(t.base(), Ty::Str(_)) && matches!(vt.base(), Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) | Ty::Bool | Ty::Enum { .. } | Ty::Brain);
            if two_texts {
                return err(Diagnostic::error(format!("say which text `to.{}` makes of {}: `to.stored.{}[…]` (what is stored) or `to.useful.{}[…]` (the useful value) (owner, 2026-10-02)", target_spelling, self.family_word(&vt), target_spelling, target_spelling)).at(span));
            }
            if matches!(t.base(), Ty::Deci(_)) && vt.is_brain() {
                return err(Diagnostic::error(format!("say which value `to.{}` takes of a brain16: `to.stored.{}[…]` (what is stored) or `to.useful.{}[…]` (the useful value) (owner, 2026-10-02)", target_spelling, target_spelling, target_spelling)).at(span));
            }
            if matches!(t.base(), Ty::Deci(_)) && matches!(vt.base(), Ty::Bin(_)) {
                return err(Diagnostic::error(format!("say which value `to.{}` takes of a bin: `to.stored.{}[…]` (what is stored) or `to.useful.{}[…]` (the useful value) (owner, 2026-10-02)", target_spelling, target_spelling, target_spelling)).at(span));
            }
        }
        let fails_owned: Vec<String> = fails.iter().map(|s| s.to_string()).collect();
        Ok(Expr { kind: K::Conv { target: t.clone(), expr: Box::new(v), otherwise: ow, fails: fails_owned, useful, bits: false }, ty: t, span })
    }

    /// `to.int8.bits[…]` (owner, 2026-10-06): a whole number's bits read as
    /// the type's — `to.int8.bits[128]` is -128, `to.uint8.bits[-1]` is 255.
    /// A value from -2^(N-1) to 2^N - 1 has them (design claude's reading,
    /// not ruled); what `.bits` means past whole numbers is not decided.
    fn to_bits(&mut self, ctx: &mut FnCtx, target: &ast::Type, operand: &ast::Expr, otherwise: Option<&ast::Otherwise>, span: Span) -> Result<Expr> {
        let mut blanks = Vec::new();
        let t = self.items.resolve_type(target, ctx.module, &mut blanks, false)?;
        let n = match &t {
            Ty::Int(IntW::Fixed { bits, .. }) if t.exact_limit().is_none() => *bits,
            _ => return err(Diagnostic::error(format!("`.bits` is for whole-number types: what `to.{}.bits` means is not decided", self.spell(&t))).at(span)),
        };
        if otherwise.is_some() {
            return err(Diagnostic::error(format!("`to.{}.bits` cannot fail: a value past its bits is a check site, not a failure; drop the `otherwise`", self.spell(&t))).at(span));
        }
        let v = self.expr(ctx, operand)?;
        let literal = match &v.kind {
            K::Int(i) => Some(*i),
            K::Unary { op: UnOp::Neg, expr } => match expr.kind {
                K::Int(i) => Some(-i),
                _ => None,
            },
            _ => None,
        };
        if let (Some(i), true) = (literal, matches!(self.resolve(&v.ty).base(), Ty::Int(IntW::Free(_)))) {
            let room = 1i128.checked_shl(n).filter(|_| n < 127);
            if let Some(room) = room {
                if i < -(room / 2) || i >= room {
                    return err(Diagnostic::error(format!("`{}` does not fit the {} bits of {}: `to.{}.bits` reads a pattern of {} bits", i, n, self.spell(&t), self.spell(&t), n)).at(v.span));
                }
                let signed = matches!(t, Ty::Int(IntW::Fixed { signed: true, .. }));
                let pattern = i.rem_euclid(room);
                let value = if signed && pattern >= room / 2 { pattern - room } else { pattern };
                return Ok(Expr { kind: K::Int(value), ty: t, span });
            }
        }
        let v = self.as_operand(ctx, v)?;
        let vt = self.resolve(&v.ty);
        if !matches!(vt.base(), Ty::Int(_)) {
            return err(Diagnostic::error(format!("`to.{}.bits` reads a whole number's bits, and this is {}", self.spell(&t), article(&self.spell(&vt)))).at(v.span));
        }
        Ok(Expr { kind: K::Conv { target: t.clone(), expr: Box::new(v), otherwise: None, fails: Vec::new(), useful: false, bits: true }, ty: t, span })
    }

    // ------------------------------------------------------------ builtins

    fn builtin_head(&self, path: &ast::Path, span: Span) -> Result<Option<BuiltinHead>> {
        let head = path.head();
        let rest: Vec<&str> = path.segs[1..].iter().map(|s| s.name.as_str()).collect();
        let b = match (head, rest.as_slice()) {
            ("list", ["length"]) => BuiltinHead::B(Builtin::ListLength),
            ("list", ["append"]) => BuiltinHead::B(Builtin::ListAppend),
            ("list", ["fill"]) => BuiltinHead::B(Builtin::ListFill),
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
            // Built by the parser from `on.machine.chip X` (owner, 2026-10-06).
            ("on", ["machine", "chip", f]) => match ast::ON_FEATURES.iter().position(|n| n == f) {
                Some(i) => BuiltinHead::B(Builtin::MachineHas(i as u32)),
                None => return Ok(None),
            },
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
            ("simd", ["shuffle"]) => BuiltinHead::B(Builtin::SimdShuffle(Vec::new())),
            ("simd", ["load", "some"]) => BuiltinHead::B(Builtin::SimdLoadSome),
            ("simd", ["store", "some"]) => BuiltinHead::B(Builtin::SimdStoreSome),
            ("simd", [other]) => return err(Diagnostic::error(format!("`simd.{}` does not exist: a vector has `fill`, `load`, `store`, `load.some`, `store.some`, `shuffle`, `all`, `any`, `max`, `min`, `sum`", other)).at(span)),
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
        let mut shuffle_order: Option<Vec<u32>> = None;
        let (hargs, ty): (Vec<Arg>, Ty) = match &b {
            Builtin::MustReport | Builtin::MustStopFork | Builtin::MustStopDone => unreachable!("made by `must`, never written"),
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
            // A list of `count` copies of one value (owner, 2026-09-30). A
            // strict file labels the count; an owned value is copied, so a
            // bare name of one says `copy`.
            Builtin::ListFill => {
                arity(2, self)?;
                let strict = self.items.strict_files.contains(&positional[1].span.file);
                let ce = match &positional[1].kind {
                    AK::Count(inner) => &**inner,
                    _ if strict => return err(Diagnostic::error("`#strict`: the count says `count`: `list.fill[0, count 10]`").at(positional[1].span)),
                    _ => positional[1],
                };
                let value_e = positional[0];
                if let AK::Name(n) = &value_e.kind {
                    if self.peek_ty(ctx, value_e).is_some_and(|t| self.resolve(&t).is_owned()) {
                        return err(Diagnostic::error(format!("`list.fill` makes copies of '{}': `copy '{}'`", n.name, n.name)).at(value_e.span));
                    }
                }
                let v = match &value_e.kind {
                    AK::Verb { expr, .. } => {
                        let Some(pt) = self.peek_ty(ctx, expr) else {
                            return err(Diagnostic::error("`list.fill` fills with a value, or a `copy` or `give` of one").at(value_e.span));
                        };
                        self.one_argument(ctx, value_e, ParamKind::Owned, &pt, "list.fill", "the value")?
                    }
                    _ => {
                        let v = self.expr(ctx, value_e)?;
                        let v = self.as_operand(ctx, v)?;
                        Arg::Value(v)
                    }
                };
                let elem = self.resolve(&self.arg_type(&v));
                let mut c = self.expr(ctx, ce)?;
                // Bare numbers with plain math between them are worked out
                // exactly, as for a declaration's value (`operators.md`):
                // `((-2^63) x 2^62) mod 9` is 4, and the product in between
                // fits nothing (check claude, seed 11045).
                if matches!(self.resolve(&c.ty).base(), Ty::Int(IntW::Free(_)) | Ty::Infer(_)) && literal_only(&c) {
                    let to = if plain_group(&c) { Ty::exact() } else { Ty::u64() };
                    retype_literal_arith(&mut c, &to);
                }
                if !self.resolve(&c.ty).is_int() {
                    return err(Diagnostic::error(format!("`list.fill`'s count is a whole number, and this is a `{}`", self.spell(&self.resolve(&c.ty)))).at(ce.span));
                }
                (vec![v, Arg::Value(c)], Ty::Seq { elem: Box::new(elem), kind: SeqKind::List })
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
            Builtin::MachineHas(_) => {
                arity(0, self)?;
                (Vec::new(), Ty::Bool)
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
                if b != Builtin::PtrIsNull && !ctx.tier.is_some_and(ast::Tier::trusts) {
                    return err(Diagnostic::error(format!("`{}` is a !check site: nothing about C's memory can be proven — write `!check` in front of the statement", name)).at(span));
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
                    // A C enum or range read from C's memory is checked as it
                    // comes in, unless `NO_FUCKING_CHECKS` (owner, 2026-09-27).
                    Builtin::PtrRead => {
                        if elem.checked_at_door(&self.items.structs, &self.items.enums) && ctx.tier == Some(ast::Tier::NoChecks) {
                            self.items.trusted_door = true;
                        }
                        if elem.checked_at_door(&self.items.structs, &self.items.enums) && ctx.tier != Some(ast::Tier::NoChecks) {
                            if otherwise.is_none() {
                                return err(Diagnostic::error(format!("`ptr.read` of a `{}` from C can fail (out_of_range): C may hold a number `{}` does not allow — say what happens with `otherwise`", self.spell(&elem), self.spell(&elem))).at(span));
                            }
                            fails.push(("out_of_range".into(), None));
                        }
                        (vec![Arg::Value(p)], elem)
                    }
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
                        // C's memory holds a C enum as its number, so C's
                        // plain number may be written there as it is
                        // (`examples/c_variables.cpls`, 2026-09-28).
                        let c_enum = matches!(elem.base(), Ty::Enum { id, .. } if self.items.enums[id.0 as usize].numbers.is_some());
                        let v = if c_enum && self.resolve(&v.ty).is_int() {
                            self.expect_store(ctx, v, &Ty::Int(IntW::Fixed { signed: true, bits: 32 }), "C's number for a C enum")?
                        } else {
                            self.expect_store(ctx, v, &elem, "what the pointer points at")?
                        };
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
                let mut v = self.expr(ctx, positional[0])?;
                // Bare numbers alone are worked out exactly (dc83c05): left
                // with an open width, `15059671820752541943 div 3` was done
                // at a signed 64 bits, where the literal is negative, in any
                // build that did not fold it (check claude, seed 5120).
                if literal_only(&v) && !is_int_literal(&v) && matches!(self.resolve(&v.ty).base(), Ty::Int(IntW::Free(_))) {
                    retype_literal_arith(&mut v, &Ty::exact());
                }
                // An answer of plain math is exact, and lands where it is
                // stored (owner, 2026-09-23): in every lane, whose type the
                // vector takes from its place. As the lane type it made a
                // vector of 65-bit lanes, one to a register — `simd.fill['x'
                // + 1]` into a `simd[int32, 4]` was a `simd[integer, 1]`
                // (check claude). MIR asks whether it fits the lane.
                if self.resolve(&v.ty).is_exact() {
                    let ty = Ty::Simd { elem: Box::new(Ty::Int(IntW::Free(InferId(0)))), lanes: 0 };
                    (vec![Arg::Value(v)], ty)
                } else {
                    // The lane type comes from where the vector is going.
                    let lane = self.fresh();
                    let v = self.expect_store(ctx, v, &lane, "every lane")?;
                    let ty = Ty::Simd { elem: Box::new(lane), lanes: 0 };
                    (vec![Arg::Value(v)], ty)
                }
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
            // The leftover end of a vector loop (owner, 2026-09-27: a).
            Builtin::SimdLoadSome | Builtin::SimdStoreSome => {
                let store = matches!(b, Builtin::SimdStoreSome);
                arity(if store { 4 } else { 3 }, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, store)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, .. } => (**elem).clone(),
                    other => return err(Diagnostic::error(format!("`{}` works on a list or an array, and {} is a `{}`", name, self.arg_name(ctx, &a), self.spell(other))).at(positional[0].span)),
                };
                let idx = self.pos_arg(ctx, positional[1], &name)?;
                let k = self.expr(ctx, positional[2])?;
                if !self.resolve(&k.ty).is_int() {
                    return err(Diagnostic::error(format!("`{}`'s third argument is how many items, a whole number from 0 to the lanes, and this is a `{}`", name, self.spell(&self.resolve(&k.ty)))).at(positional[2].span));
                }
                if !store {
                    (vec![a, idx, Arg::Value(k)], Ty::Simd { elem: Box::new(elem), lanes: 0 })
                } else {
                    let v = self.expr(ctx, positional[3])?;
                    let vt = self.resolve(&v.ty);
                    match vt.base() {
                        Ty::Simd { elem: lane, .. } => {
                            if !self.unify(lane, &elem) {
                                return err(Diagnostic::error(format!("`simd.store.some` writes `{}` lanes into a list of `{}`", self.spell(lane), self.spell(&elem))).at(positional[3].span));
                            }
                        }
                        other => return err(Diagnostic::error(format!("`simd.store.some` writes a vector, and this is a `{}`", self.spell(other))).at(positional[3].span)),
                    }
                    (vec![a, idx, Arg::Value(k), Arg::Value(v)], Ty::Nothing)
                }
            }
            // One rearrangement for all (owner, 2026-09-27: a): the order is
            // lane numbers fixed when compiling, 1-based, repeats allowed.
            Builtin::SimdShuffle(_) => {
                arity(2, self)?;
                let v = self.expr(ctx, positional[0])?;
                let v = self.as_operand(ctx, v)?;
                let vt = self.resolve(&v.ty);
                let lanes = match vt.base() {
                    Ty::Simd { lanes, .. } if *lanes > 0 => *lanes,
                    Ty::Simd { .. } => return err(Diagnostic::error("`simd.shuffle` needs to know how many lanes the vector has: give it a type (`let 'v': simd[int32, 4] = …`)").at(positional[0].span)),
                    other => return err(Diagnostic::error(format!("`simd.shuffle` rearranges a vector's lanes, and this is a `{}`", self.spell(other))).at(positional[0].span)),
                };
                let fixed = "`simd.shuffle` takes the new order as fixed lane numbers, known when compiling: `[4, 3, 2, 1]`";
                let AK::List(items) = &positional[1].kind else {
                    return err(Diagnostic::error(fixed).at(positional[1].span));
                };
                let mut order = Vec::new();
                for it in items {
                    let AK::Int(text) = &it.kind else {
                        return err(Diagnostic::error(fixed).at(it.span));
                    };
                    let n: u64 = text.parse().unwrap_or(u64::MAX);
                    if n < 1 || n > lanes as u64 {
                        return err(Diagnostic::error(format!("lane {} does not exist: the vector has {} lanes, 1 to {}", text, lanes, lanes)).at(it.span));
                    }
                    order.push(n as u32 - 1);
                }
                if order.len() != lanes as usize {
                    return err(Diagnostic::error(format!("the new order names {} lane(s), and the vector has {}: one lane number per lane", order.len(), lanes)).at(positional[1].span));
                }
                shuffle_order = Some(order);
                (vec![Arg::Value(v)], vt.base().clone())
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
        let b = match shuffle_order {
            Some(order) => Builtin::SimdShuffle(order),
            None => b,
        };
        let ow = self.call_otherwise(ctx, otherwise, &fails, &ty, used, span, &name)?;
        self.lends_apart(ctx, &hargs, span)?;
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
                self.items.structs.push(StructInfo { name: name.into(), path: name.into(), layout_c: false, fields: vec![FieldInfo { name: "key".into(), immut: true, ty: kt }, FieldInfo { name: "value".into(), immut: true, ty: vt }], blanks: Vec::new(), bounds: Vec::new(), span: Span::new(crate::source::FileId(0), 0, 0) });
                id
            }
        };
        Ty::Struct { id, args: Vec::new() }
    }

    /// A builtin's collection argument: `lend 'xs'` (or `lend.mut` when it changes it).
    fn lent_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr, name: &str, mutates: bool) -> Result<Arg> {
        match &e.kind {
            AK::Verb { verb, expr, .. } => {
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
    /// A position, written bare (owner, 2026-09-27: `pos` is removed).
    fn pos_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr, name: &str) -> Result<Arg> {
        // A strict file says every position (owner, 2026-09-27).
        let e = match &e.kind {
            AK::Pos(inner) => &**inner,
            _ if self.items.strict_files.contains(&e.span.file) => return err(Diagnostic::error(format!("`{}` says each position with `pos` — `pos 2` — in a `#strict` file", name)).at(e.span)),
            _ => e,
        };
        let v = self.expr(ctx, e)?;
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
        Builtin::ListLength | Builtin::ListAppend | Builtin::ListRemove | Builtin::ListSlice | Builtin::ListFill => "list",
        Builtin::ArrayLength | Builtin::ArraySlice => "array",
        Builtin::MapLength | Builtin::MapHas | Builtin::MapGet | Builtin::MapSet | Builtin::MapRemove | Builtin::MapKeys | Builtin::MapValues | Builtin::MapPairs => "map",
        Builtin::StrBytes | Builtin::StrPoints | Builtin::StrGraphemes | Builtin::StrEach(_) => "text",
        _ => "value",
    }
}

/// Bare numbers joined only by plain math (`+ - x xx`), which is exact.
/// Bare numbers with only plain math between them: `+ - x xx`, and `/` and
/// `mod`, which on true values give true values — "bare numbers alone are
/// worked out exactly" (`operators.md`). `(2^64 - 1 + 1) mod 1` is 0; with
/// `mod` left out it was worked at the place's width and refused (check
/// claude, attack a). Shifts, bitwise and wrapping take the place's width.
fn plain_group(e: &Expr) -> bool {
    match &e.kind {
        K::Binary { op, lhs, rhs, .. } => matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::Mod(_)) && plain_group(lhs) && plain_group(rhs),
        K::Unary { op: UnOp::Neg, expr } => plain_group(expr),
        K::Int(_) | K::BigInt(_) => true,
        _ => false,
    }
}

/// Bare numbers — whole or with a point — with only `+ - x div` between
/// them: what a `deci` place makes decimals of.
/// The width of an exact literal (`*0.1`) not yet given by its place.
pub const EXACT_FREE: u16 = 0;

/// An exact literal no place gave a width: `deci64`, the oracle's pick
/// (provisional: the width is Precog's to choose).
pub fn settle_exact(e: &mut Expr) -> Result<()> {
    if matches!(e.ty.base(), Ty::Deci(EXACT_FREE)) {
        into_deci(e, 64)?;
    }
    Ok(())
}

fn deci_group(e: &Expr) -> bool {
    match &e.kind {
        K::Int(_) => matches!(e.ty.base(), Ty::Int(IntW::Free(_))) || e.ty.is_exact(),
        K::Bin(_) => matches!(e.ty.base(), Ty::Bin(BinW::Free(_)) | Ty::Deci(EXACT_FREE)),
        K::Unary { op: UnOp::Neg, expr } => deci_group(expr),
        K::Binary { op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(Round::Near), lhs, rhs, .. } => deci_group(lhs) && deci_group(rhs),
        _ => false,
    }
}

/// A `deci_group` made decimals of `bits`: each literal exactly as
/// written, and one with more digits than the width holds refused, never
/// rounded (`types.md`).
fn into_deci(e: &mut Expr, bits: u16) -> Result<()> {
    let text = match &e.kind {
        K::Int(i) => Some(i.to_string()),
        K::Bin(t) => Some(t.clone()),
        _ => None,
    };
    if let Some(text) = text {
        let p = crate::deci::digits(bits);
        let value = match crate::deci::literal(&text, bits) {
            Ok(v) => v,
            Err(crate::deci::LitErr::Digits(n)) => return err(Diagnostic::error(format!("`{}` has {} digits; deci{} holds {}: write fewer digits, or use a wider deci", text, n, bits, p)).at(e.span)),
            Err(crate::deci::LitErr::Range) => return err(Diagnostic::error(format!("`{}` is past what deci{} holds: write fewer digits, or use a wider deci", text, bits)).at(e.span)),
        };
        e.kind = K::Deci { bits, value };
    } else {
        match &mut e.kind {
            K::Unary { expr, .. } => into_deci(expr, bits)?,
            K::Binary { lhs, rhs, .. } => {
                into_deci(lhs, bits)?;
                into_deci(rhs, bits)?;
            }
            _ => {}
        }
    }
    e.ty = Ty::Deci(bits);
    Ok(())
}

/// Every leaf a literal, every node arithmetic.
impl Checker {
    /// A struct's or enum's name with the Limb it comes from: `areas::Circle`.
    fn spell_with_limb(&self, t: &Ty) -> String {
        let path = match t.base() {
            Ty::Struct { id, .. } => self.items.structs[id.0 as usize].path.clone(),
            Ty::Enum { id, .. } => self.items.enums[id.0 as usize].path.clone(),
            _ => return self.spell(t),
        };
        let path = path.replace("::limb.", "::");
        path.strip_prefix("limb.").map(|p| format!("my.{}", p)).unwrap_or(path)
    }
}

/// One of std's names the oracle stands in for (owner, 2026-10-06: std
/// is `bare.optional` for all), written without `std::`.
fn std_name(path: &ast::Path) -> bool {
    let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
    match segs.as_slice() {
        ["print" | "printu" | "printf", ..] => true,
        ["exit" | "round" | "floor" | "ceil" | "trunc"] => true,
        ["time" | "input", _, ..] => true,
        ["str", op] => matches!(*op, "split" | "find" | "contains" | "trim" | "upper" | "lower" | "replace" | "starts_with" | "ends_with" | "at" | "slice" | "join"),
        _ => false,
    }
}

/// std's lists, maps and text units, which the language builds in, with
/// `std::` in front.
fn std_builtin(path: &ast::Path) -> bool {
    let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
    match segs.as_slice() {
        ["list" | "map", _, ..] => true,
        ["str", unit, ..] => matches!(*unit, "bytes" | "points" | "graphemes" | "each"),
        _ => false,
    }
}

fn literal_only(e: &Expr) -> bool {
    match &e.kind {
        K::Int(_) | K::BigInt(_) => true,
        K::Unary { op: crate::ast::UnOp::Neg, expr } => literal_only(expr),
        // Every operator that gives a whole number of its operands' width:
        // arithmetic, shifts, bitwise, wrapping.
        K::Binary { op, lhs, rhs, .. } => {
            use crate::ast::BinOp as B;
            matches!(op, B::Add | B::Sub | B::Mul | B::Pow | B::Div(_) | B::Mod(_) | B::Shl | B::Shr | B::AndBit | B::OrBit | B::Xor | B::WrapAdd | B::WrapSub | B::WrapMul | B::WrapPow) && literal_only(lhs) && literal_only(rhs)
        }
        K::Unary { op: crate::ast::UnOp::NotBit, expr } => literal_only(expr),
        _ => false,
    }
}

/// Give every arithmetic node of a literal-only expression the place's type.
/// The leaves keep their values; a leaf the type cannot hold is then a
/// narrowing Precog refuses as certain.
/// A group of bare numbers worked out exactly: the bits its largest value
/// on the way takes, signed; `None` where it has no answer (a division by
/// zero, a negative power) or one past every width there is.
fn literal_group_bits(e: &Expr) -> Option<u32> {
    fn eval(e: &Expr, most: &mut u64) -> Option<crate::bigint::Big> {
        use crate::bigint::Big;
        let v = match &e.kind {
            K::Int(v) => Big::from_i128(*v),
            K::BigInt(d) => Big::parse(d)?,
            K::Unary { op: UnOp::Neg, expr } => eval(expr, most)?.neg(),
            K::Binary { op, lhs, rhs, .. } => {
                let (a, b) = (eval(lhs, most)?, eval(rhs, most)?);
                match op {
                    BinOp::Add => a.add(&b),
                    BinOp::Sub => a.sub(&b),
                    BinOp::Mul => a.mul(&b),
                    BinOp::Pow => {
                        let k = b.to_i128().filter(|k| *k >= 0)?;
                        a.pow(u64::try_from(k).ok()?)?
                    }
                    BinOp::Div(r) => a.divmod(&b, *r)?.0,
                    BinOp::Mod(r) => a.divmod(&b, *r)?.1,
                    _ => return None,
                }
            }
            _ => return None,
        };
        *most = (*most).max(v.bits());
        (v.bits() <= crate::types::WIDEST_INT as u64).then_some(v)
    }
    let mut most = 0;
    eval(e, &mut most)?;
    Some(most as u32 + 1)
}

/// The bits the biggest bare number in an expression takes, signed.
fn literal_bits(e: &Expr) -> u32 {
    match &e.kind {
        K::Int(v) => 129 - v.unsigned_abs().leading_zeros(),
        K::BigInt(d) => crate::bigint::Big::parse(d).map_or(0, |b| b.bits() as u32 + 1),
        K::Unary { expr, .. } => literal_bits(expr),
        K::Binary { lhs, rhs, .. } => literal_bits(lhs).max(literal_bits(rhs)),
        _ => 0,
    }
}

/// Each exact operation with a bare number for an operand, down an exact
/// answer's tree, held to at least `bits` in between.
fn raise_beside_literal(e: &mut Expr, bits: u32) {
    let Some(limit) = e.ty.exact_limit() else { return };
    match &mut e.kind {
        K::Binary { lhs, rhs, .. } => {
            if (literal_only(lhs) || literal_only(rhs)) && bits > limit {
                e.ty = Ty::exact_at(bits);
            }
            raise_beside_literal(lhs, bits);
            raise_beside_literal(rhs, bits);
        }
        // A negative of one is worked at its operand's limit.
        K::Unary { expr, .. } => {
            raise_beside_literal(expr, bits);
            if let Some(inner) = expr.ty.exact_limit() {
                if inner > limit {
                    e.ty = Ty::exact_at(inner);
                }
            }
        }
        _ => {}
    }
}

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
        K::Int(_) | K::BigInt(_) => true,
        K::Unary { op: crate::ast::UnOp::Neg, expr } => matches!(expr.kind, K::Int(_) | K::BigInt(_)),
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

/// A bin with no width yet (a bare literal, a `:=` of one) takes the width
/// of the typed value beside it, and the answer is at that width (owner,
/// 2026-09-24: a; `operators.md`): `'x' + 1.0` on a bin32 is bin32 math.
/// Two widths: the wider (the oracle's pick, provisional).
pub fn join_bin(a: BinW, b: BinW) -> BinW {
    match (a, b) {
        (BinW::Free(_), BinW::Fixed(x)) | (BinW::Fixed(x), BinW::Free(_)) => BinW::Fixed(x),
        (BinW::Free(_), BinW::Free(_)) => BinW::Free(InferId(0)),
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
            "print.stored.stdout" | "print.stored.stderr" | "print.useful.stdout" | "print.useful.stderr" | "print.weird.stored.stdout" | "print.weird.stored.stderr" | "print.weird.useful.stdout" | "print.weird.useful.stderr" | "print.useful.auto_newline.stdout" | "print.useful.auto_newline.stderr" | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_output" | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_error" | "print.format.stdout" | "print.format.stderr" => {
                for e in &positional {
                    // A print only looks: a borrow handed back may go
                    // straight to it.
                    let saved = ctx.lend_call_ok.take();
                    if let AK::Call { .. } = &e.kind {
                        ctx.lend_call_ok = Some(e.span);
                    }
                    let v = self.expr(ctx, e);
                    ctx.lend_call_ok = saved;
                    let v = v?;
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

/// `have` with every number whose width is not settled — a bare number's,
/// or the exact one plain math gives — taking `want`'s width, where `want`
/// has one; `None` when nothing changes.
pub fn adopt_widths(have: &Ty, want: &Ty) -> Option<Ty> {
    fn go(h: &Ty, w: &Ty, changed: &mut bool) -> Ty {
        match (h, w) {
            (Ty::Int(IntW::Free(_)), Ty::Int(IntW::Fixed { .. })) => {
                *changed = true;
                w.clone()
            }
            (Ty::Int(IntW::Fixed { .. }), Ty::Int(IntW::Fixed { .. })) if h.is_exact() && !w.is_exact() => {
                *changed = true;
                w.clone()
            }
            (Ty::Bin(crate::types::BinW::Free(_)), Ty::Bin(crate::types::BinW::Fixed(_))) => {
                *changed = true;
                w.clone()
            }
            (Ty::Seq { elem: e1, kind }, Ty::Seq { elem: e2, .. }) => Ty::Seq { elem: Box::new(go(e1, e2, changed)), kind: *kind },
            // "A lend satisfies a pointer" (`ffi.md`): C reads the elements.
            (Ty::Seq { elem: e1, kind }, Ty::Ptr { elem: e2, .. }) => Ty::Seq { elem: Box::new(go(e1, e2, changed)), kind: *kind },
            (Ty::Map { key: k1, val: v1, order }, Ty::Map { key: k2, val: v2, .. }) => Ty::Map { key: Box::new(go(k1, k2, changed)), val: Box::new(go(v1, v2, changed)), order: *order },
            (Ty::Channel(e1), Ty::Channel(e2)) => Ty::Channel(Box::new(go(e1, e2, changed))),
            (Ty::Bounded { inner, bound }, w) => Ty::Bounded { inner: Box::new(go(inner, w.base(), changed)), bound: bound.clone() },
            _ => h.clone(),
        }
    }
    let mut changed = false;
    let t = go(have, want.base(), &mut changed);
    changed.then_some(t)
}

/// Two types that are the same memory: every number the same width, all
/// the way down. Bounds change no layout; a width not written yet and a
/// blank `<T>` match anything.
pub fn sizes_match(a: &Ty, b: &Ty) -> bool {
    match (a.base(), b.base()) {
        (Ty::Int(IntW::Free(_)), Ty::Int(_)) | (Ty::Int(_), Ty::Int(IntW::Free(_))) => true,
        (Ty::Int(x), Ty::Int(y)) => x == y,
        (Ty::Bin(crate::types::BinW::Free(_)), Ty::Bin(_)) | (Ty::Bin(_), Ty::Bin(crate::types::BinW::Free(_))) => true,
        (Ty::Bin(x), Ty::Bin(y)) => x == y,
        (Ty::Seq { elem: x, .. }, Ty::Seq { elem: y, .. }) | (Ty::Channel(x), Ty::Channel(y)) | (Ty::Ptr { elem: x, .. }, Ty::Ptr { elem: y, .. }) => sizes_match(x, y),
        (Ty::Map { key: k1, val: v1, .. }, Ty::Map { key: k2, val: v2, .. }) => sizes_match(k1, k2) && sizes_match(v1, v2),
        (Ty::Struct { args: x, .. }, Ty::Struct { args: y, .. }) | (Ty::Enum { args: x, .. }, Ty::Enum { args: y, .. }) => x.len() != y.len() || x.iter().zip(y).all(|(p, q)| sizes_match(p, q)),
        _ => true,
    }
}

/// `to.stored.T` / `to.useful.T`: the form named, and the target without
/// it — inside a bound too (`to.stored.str.utf8[0, 3 bytes][…]`).
fn conv_form(target: &ast::Type) -> Result<(Option<bool>, Option<ast::Type>)> {
    match &target.kind {
        ast::TypeKind::Path { path, .. } if path.segs.len() >= 2 && path.segs.last().is_some_and(|s| s.name == "useful" || s.name == "stored") => {
            let rest: Vec<&str> = path.segs[..path.segs.len() - 1].iter().map(|s| s.name.as_str()).collect();
            let f = &path.segs.last().unwrap().name;
            err(Diagnostic::error(format!("`.{}` follows the verb: `to.{}.{}[…]` (owner, 2026-09-30)", f, f, rest.join("."))).at(target.span))
        }
        ast::TypeKind::Path { path, args } if path.segs.len() >= 2 && (path.segs[0].name == "useful" || path.segs[0].name == "stored") => {
            let mut p = path.clone();
            p.segs.remove(0);
            Ok((Some(path.segs[0].name == "useful"), Some(ast::Type { kind: ast::TypeKind::Path { path: p, args: args.clone() }, span: target.span })))
        }
        ast::TypeKind::Bound { inner, lo, hi, unit, span } => {
            let (form, inner2) = conv_form(inner)?;
            Ok((form, inner2.map(|i| ast::Type { kind: ast::TypeKind::Bound { inner: Box::new(i), lo: lo.clone(), hi: hi.clone(), unit: unit.clone(), span: *span }, span: target.span })))
        }
        _ => Ok((None, None)),
    }
}

/// Bare numbers with plain math between them: what a `brain16` beside
/// them, or a `brain16` place, makes brain16s.
fn bare_number_group(e: &Expr) -> bool {
    match &e.kind {
        K::Int(_) => matches!(e.ty.base(), Ty::Int(IntW::Free(_))),
        K::Bin(_) => matches!(e.ty.base(), Ty::Bin(BinW::Free(_))),
        K::Unary { op: UnOp::Neg, expr } => bare_number_group(expr),
        K::Binary { op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(Round::Near), lhs, rhs, .. } => bare_number_group(lhs) && bare_number_group(rhs),
        _ => false,
    }
}

/// A `bare_number_group` made brain16s: each literal rounds to the
/// nearest brain16 where it is built, and each step rounds (owner,
/// 2026-10-02: a).
fn into_brain(e: &mut Expr) {
    match &mut e.kind {
        K::Int(i) => e.kind = K::Bin(i.to_string()),
        K::Unary { expr, .. } => into_brain(expr),
        K::Binary { lhs, rhs, .. } => {
            into_brain(lhs);
            into_brain(rhs);
        }
        _ => {}
    }
    e.ty = Ty::Brain;
}
