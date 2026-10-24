//! Expressions, places, calls, verbs, conversions, `otherwise`.

use crate::ast::{self, Arg as AArg, BinOp, ExprKind as AK, Round, UnOp, Verb};
use crate::diag::{err, Diagnostic, Edit, Result};
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
                return err(Diagnostic::error(format!("{}[…] on its own reads an item and drops it; to change it, `change {}(…) = …;`", who, who)).named("value_left_unused").at(e.span));
            }
            _ => return err(Diagnostic::error("a value alone is not a statement: assign it, or call something with it").named("value_left_unused").at(e.span)),
        }
        Ok(v)
    }

    fn expr_in(&mut self, ctx: &mut FnCtx, e: &ast::Expr, top: bool, used: bool) -> Result<Expr> {
        let span = e.span;
        // A bare enum value where the type is known (owner, 2026-10-20):
        // `say[calm]` for `'m': Mood`, when `Mood` lets its values drop its
        // name. Only this expression is asked, not what is inside it.
        let wanted = self.want.take();
        if let Some(want) = &wanted {
            if let Some(r) = self.bare_case(ctx, e, want) {
                return r;
            }
        }
        match &e.kind {
            // `80%`, `exactly 7%` (owner, 2026-10-17): in a plain number's
            // place, the number over 100; anywhere else, a percent of it.
            AK::Percent(lit) => {
                let v = self.expr(ctx, lit)?;
                if self.plain_numbers {
                    return Ok(v);
                }
                let ty = Ty::Percent(Box::new(v.ty.clone()));
                Ok(Expr { ty, ..v })
            }
            AK::Int(t) => {
                if let Ok(v) = t.parse::<i128>() {
                    return Ok(Expr { kind: K::Int(v), ty: FREE_INT, span });
                }
                // Past `i128`: kept as its digits, for a wide type to hold
                // (owner, 2026-10-03), and past the widest there is, refused.
                match crate::bigint::Big::parse(t) {
                    Some(b) if b.bits() <= crate::types::WIDEST_INT as u64 => Ok(Expr { kind: K::BigInt(b.to_string()), ty: FREE_INT, span }),
                    _ => err(Diagnostic::error(format!("`{}` is too large for any integer type: the widest there is holds {} bits", t, crate::types::WIDEST_INT)).named("value_does_not_fit").at(span)),
                }
            }
            AK::Bin(t) => Ok(Expr { kind: K::Bin(t.clone()), ty: FREE_BIN, span }),
            // A deci whose width its place gives: `d64` under `:=`.
            AK::Exact(t) => Ok(Expr { kind: K::Bin(t.clone()), ty: Ty::Deci(EXACT_FREE), span }),
            AK::Typed { value, ty } => self.typed_literal(ctx, value, ty, span),
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
            AK::Duration { unit, .. } if unit.name == "bytes" => err(Diagnostic::error("a size in bytes is a writer's: `std::print.writer(size: 4096 bytes)`").named("bound_unit_misplaced").at(span)),
            AK::Duration { value, unit } => {
                let v: i128 = value.parse().map_err(|_| Box::new(Diagnostic::error("this duration is too large").named("value_does_not_fit").at(span)))?;
                let nanos = match unit.name.trim_end_matches('s') {
                    "nano" => 1,
                    "micro" => 1_000,
                    "milli" => 1_000_000,
                    "sec" => 1_000_000_000,
                    "min" => 60_000_000_000,
                    "hour" => 3_600_000_000_000i128,
                    _ => unreachable!(),
                };
                Ok(Expr { kind: K::Duration(v * nanos), ty: Ty::Duration, span })
            }
            AK::Name(n) => {
                // A const declared in this body (owner, 2026-10-10).
                if let Some(cid) = ctx.lookup_const(&n.name) {
                    let ty = self.const_type(cid, span)?;
                    return Ok(Expr { kind: K::Const(cid), ty, span });
                }
                // A body const's value uses no variable.
                if ctx.lookup(&n.name).is_none() && ctx.outer_vars.as_ref().is_some_and(|v| v.contains(&n.name)) {
                    return err(Diagnostic::error(format!("a const's value must be known at compile time; '{}' is a variable (use `let` for a value worked out while running)", n.name)).named("const_not_known").at(span));
                }
                let Some(id) = ctx.lookup(&n.name) else {
                    // A const of this module, or C's variable.
                    if let Some((cid, _, _)) = self.items.modules[ctx.module.0 as usize].const_named(&n.name).cloned() {
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
                    return err(Diagnostic::error("a call that can fail must be the whole value: put it in its own declaration, then use the result").named("otherwise_misplaced").at(span));
                }
                self.call(ctx, path, args, otherwise.as_deref(), span, used)
            }
            AK::CallValue { callee, args, otherwise } => {
                if otherwise.is_some() && !top {
                    return err(Diagnostic::error("a call that can fail must be the whole value: put it in its own declaration, then use the result").named("otherwise_misplaced").at(span));
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
                    return err(Diagnostic::error("a conversion that can fail must be the whole value: put it in its own declaration, then use the result").named("otherwise_misplaced").at(span));
                }
                self.conversion(ctx, target, expr, otherwise.as_deref(), span, used)
            }
            AK::Channel { elem, bound } => {
                let et = self.body_type(ctx, elem)?;
                let cap = match bound {
                    Some(b) => {
                        let (lo, hi) = (&b.0, &b.1);
                        let l = crate::resolve::int_literal(lo).ok_or_else(|| Box::new(Diagnostic::error("a channel's bound is written as numbers: `channel[T][0, 100]`").named("mcps::channel_form").at(lo.span)))?;
                        let h = crate::resolve::int_literal(hi).ok_or_else(|| Box::new(Diagnostic::error("a channel's bound is written as numbers: `channel[T][0, 100]`").named("mcps::channel_form").at(hi.span)))?;
                        if l < 0 || h < l {
                            return err(Diagnostic::error("a channel's bound is `[0, N]`: the low end 0, the high end how many values may wait").named("mcps::channel_form").at(span));
                        }
                        Some((l as u64, h as u64))
                    }
                    None => None,
                };
                Ok(Expr { kind: K::Channel { elem: et.clone(), capacity: cap }, ty: Ty::Channel(Box::new(et)), span })
            }
            AK::FnLit(lit) => self.fn_literal(ctx, lit, span, wanted.as_ref()),
            AK::Spread { .. } => err(Diagnostic::error("`...` has no meaning here: it unpacks a list into a call's arguments — `sum(...$xs)`").named("spread_outside_call").at(span)),
            AK::Incl { form, path, path_span } => self.incl(ctx, *form, path, *path_span, span),
            AK::SizeOf { align, ty } => {
                // The function's own generics are announced, so naming one
                // here is no typo — it just has no size or layout yet.
                let mut blanks: Vec<(String, Vec<crate::types::IfaceId>)> = ctx.blank_names.iter().map(|n| (n.clone(), Vec::new())).collect();
                let t = self.items.resolve_type_opaque_ok(ty, ctx.module, &mut blanks, true)?;
                if t.contains_var() {
                    return err(Diagnostic::error("a blank has no size until it is filled: `size.of` takes a concrete type").named("generic_size_unknown").at(ty.span));
                }
                if let Ty::Struct { id, .. } = t.base() {
                    let info = &self.items.structs[id.0 as usize];
                    if info.opaque {
                        return err(Diagnostic::error(format!("`{}` is C's and opaque: its size is C's own, and CyborgPL only points at it", info.name)).named("cnc::opaque_struct_held").at(ty.span));
                    }
                }
                Ok(Expr { kind: K::SizeOf { align: *align, ty: t }, ty: Ty::u64(), span })
            }
            AK::PtrCast { ty, value } => {
                self.power_use(ctx, &["raw_memory", "cast"], None, "casts raw memory (`ptr.cast`)".into(), span);
                // Nothing proves what the bytes are (owner, 2026-09-30).
                if self.needs_mark(ctx) {
                    self.power_first()?;
                    return err(Diagnostic::error("`ptr.cast` says the bytes are a different type, which the compiler cannot vouch for — it is a no_check site: write `no_check` in front of the statement").named("no_check_missing").at(span).fix_edit(span, Edit::Before, "no_check "));
                }
                // The function's own generics are announced, so naming one
                // here is no typo — it just has no size or layout yet.
                let mut blanks: Vec<(String, Vec<crate::types::IfaceId>)> = ctx.blank_names.iter().map(|n| (n.clone(), Vec::new())).collect();
                let t = self.items.resolve_type(ty, ctx.module, &mut blanks, true)?;
                if t.contains_var() {
                    return err(Diagnostic::error("`ptr.cast` takes a concrete type: a blank has no layout").named("pointer_cast_target").at(ty.span));
                }
                if !t.is_c_shaped(&self.items.structs, &self.items.enums) {
                    let why = match t.base() {
                        Ty::Struct { id, .. } => format!("struct `{}` is not `layout.C`; its layout is the prover's, not C's", self.items.structs[id.0 as usize].name),
                        Ty::Enum { id, .. } => format!("enum `{}` is not `layout.C`; its layout is the prover's, not C's", self.items.enums[id.0 as usize].name),
                        other => format!("`{}` is not C-shaped", self.spell(other)),
                    };
                    return err(Diagnostic::error(format!("a pointer points at C-shaped data, so it casts only to a C-shaped type: {}", why)).named("pointer_cast_target").at(ty.span));
                }
                let v = self.expr(ctx, value)?;
                let vt = self.resolve(&v.ty);
                // Whether it may write stays as it was (owner, 2026-09-30: a).
                let mutable = match vt.base() {
                    Ty::Ptr { mutable, .. } => *mutable,
                    other => return err(Diagnostic::error(format!("`ptr.cast` takes a pointer, and this is {}: a pointer and a number never turn into each other", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(value.span)),
                };
                Ok(Expr { kind: K::PtrCast(Box::new(v)), ty: Ty::Ptr { mutable, elem: Box::new(t) }, span })
            }
            AK::Count(_) => err(Diagnostic::error("`count` labels `list.fill`'s count: `list.fill(value: 0, count: 10)`").named("label_outside_call").at(span)),
            AK::Value(_) => err(Diagnostic::error("`value` labels what a fill fills with: `list.fill(value: 0, count: 10)`, `simd.fill(value: 0)`").named("label_outside_call").at(span)),
            AK::Label(w, _) => err(Diagnostic::error(format!("`{}:` labels an argument of a function whose collection gives its parameters labels: `say(2, value: \"hi\")` (owner, 2026-10-21)", w.name)).named("label_outside_call").at(span)),
            AK::Pos(_) => err(Diagnostic::error("`pos` says a position, and one only goes where a list is read, sliced or loaded: `$xs[pos: 2]`, `simd.load(lend $xs, pos: $i)`").named("label_outside_call").at(span)),
            AK::Match { value, arms } => self.match_value(ctx, value, arms, span),
            AK::OpValue { spelled, .. } => err(Diagnostic::error(format!("`op {}` alone does not say which `{}`: hand it where a function is wanted — a parameter, or `let $f: fn[i32, i32] -> i32 = op {};`", spelled, spelled, spelled)).named("operator_value_ambiguous").at(span)),
            AK::Range { .. } => err(Diagnostic::error("`range[…]` is only decided as what a `for` runs over; as a value on its own it is not").named("range_as_value").at(span)),
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
                    info.fields.iter().find(|f| self.items.same_name(&self.items.case_fields, info.span, &f.name, &field.name))?.ty.subst(args)
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
                return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared inside this function: it sees nothing of the surrounding scope unless `with [lend ${}]` (or `copy` / `give`) says so", n.name, n.name)).named("not_declared").at(n.span), &n.name, n.span));
            }
        }
        if ctx.is_task {
            return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared inside this task: a task takes what it needs through `with [lend ${}]` (or `copy` / `give`)", n.name, n.name)).named("not_declared").at(n.span), &n.name, n.span));
        }
        Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared", n.name)).named("not_declared").at(n.span), &n.name, n.span))
    }

    fn const_type(&mut self, id: hir::ConstId, span: Span) -> Result<Ty> {
        match &self.items.consts[id.0 as usize].ty {
            Some(t) => Ok(t.clone()),
            None => err(Diagnostic::error(format!("'{}' is used before its value is known: consts are evaluated in order, top to bottom", self.items.consts[id.0 as usize].name)).named("const_not_known").at(span)),
        }
    }

    /// A bare path as a value: a const, a function, an enum case, `list.empty`.
    fn path_value(&mut self, ctx: &mut FnCtx, path: &ast::Path, span: Span) -> Result<Expr> {
        // `ptr.null`, C's NULL (owner, 2026-10-21): a pointer to what its
        // place says, `ptr` or `ptr.mut`.
        if path.is(&["ptr", "null"]) {
            let elem = self.fresh();
            return Ok(Expr { kind: K::Call { callee: Callee::Builtin(Builtin::PtrNull), args: Vec::new(), otherwise: None, fails: Vec::new() }, ty: Ty::Ptr { mutable: false, elem: Box::new(elem) }, span });
        }
        // std's values take its path outside std (owner, 2026-10-22):
        // `std::list.empty`, `std::map.empty`.
        let via_std = std::mem::take(&mut self.via_std);
        if path.kit.as_ref().is_some_and(|k| k.name == "std") && std_builtin(path) {
            let plain = ast::Path { kit: None, segs: path.segs.clone(), span: path.span };
            self.via_std = true;
            return self.path_value(ctx, &plain, span);
        }
        if path.kit.is_none() && std_builtin(path) && !via_std && !self.in_std(ctx) && self.items.lookup_plain(path, ctx.module).is_err() {
            return err(Diagnostic::error(std_bare_message(path)).named("std_name_needs_path").at(path.span).fix_edit(path.span, Edit::Insert, "std::"));
        }
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
            return err(Diagnostic::error("`list.append` hands nothing back and cannot be used as a value").named("call_gives_nothing").at(span));
        }
        if path.segs.len() == 1 && path.kit.is_none() {
            // A bare word: a const of this module? (consts are quoted; a bare word is never one.)
            let w = path.head();
            if crate::parser::is_keyword(w) {
                return err(Diagnostic::error(format!("`{}` is a keyword, not a value", w)).named("not_a_value").at(span));
            }
        }
        // A std name without its brackets: what std offers is called, and a
        // call needs `[…]` — the oracle's reason, before whether std is
        // there at all.
        // std's own enum values are values (`std::casing.tr`, owner,
        // 2026-10-20).
        let std_enum = path.segs.len() >= 2 && {
            let prefix = ast::Path { kit: path.kit.clone(), segs: path.segs[..path.segs.len() - 1].to_vec(), span: path.span };
            matches!(self.items.lookup_path(&prefix, ctx.module), Ok(ItemRef::Enum(_)))
        };
        if path.kit.as_ref().map_or(false, |l| l.name == "std") && !std_enum {
            return err(Diagnostic::error(format!("`{}` is called, and a call needs `(…)`: `{}(…)`", path.spelled(), path.spelled())).named("call_without_parens").at(span).fix_edit(crate::source::Span::new(span.file, span.end as usize, span.end as usize), Edit::Insert, "()"));
        }
        if let Some(id) = self.items.lookup_type_fn(path, ctx.module)? {
            let name = &self.items.fns[id.0 as usize].name;
            return err(Diagnostic::error(format!("`{}` is a function: call it, `{}(…)`", name, name)).named("call_without_parens").at(span).fix_edit(crate::source::Span::new(span.file, span.end as usize, span.end as usize), Edit::Insert, "()"));
        }
        // An enum case without data: `Status.empty`.
        if path.segs.len() >= 2 {
            let prefix = ast::Path { kit: path.kit.clone(), segs: path.segs[..path.segs.len() - 1].to_vec(), span: path.span };
            if let Ok(ItemRef::Enum(id)) = self.items.lookup_path(&prefix, ctx.module) {
                self.enum_named_ok(id, path.last(), span)?;
                return self.enum_case(ctx, id, path.last(), None, span);
            }
        }
        match self.items.lookup_path(path, ctx.module) {
            Ok(ItemRef::Fn(id)) => {
                let decl = &self.items.fns[id.0 as usize];
                if decl.op.is_some() {
                    return err(Diagnostic::error(format!("`{}` is an operator; it goes between two values and is not a value itself", decl.name)).named("operator_value_form").at(span));
                }
                // Which version would be held is not decided (design
                // claude's reading, 2026-10-20), as an operator's is not.
                if self.items.versions.contains_key(&id) {
                    return err(Diagnostic::error(format!("`{}` has several versions, so it is no single value to hold: wrap the one you mean in a function", decl.name)).named("not_a_value").at(span));
                }
                if decl.test {
                    return err(Diagnostic::error(format!("`{}` is a `test fn`: `kayo test` runs it, and it is not a value to hold", decl.name)).named("test_fn_misused").at(span));
                }
                if !decl.blanks.is_empty() || decl.params.iter().any(|p| p.ty.is_none() || p.variadic) {
                    return err(Diagnostic::error(format!("`{}` has blanks, so it has no single type to hold: pass it where the blanks are filled, or write a version with the types", decl.name)).named("not_a_value").at(span));
                }
                let key = InstKey { fn_id: id, blanks: Vec::new(), bundle: None, owning: Vec::new() };
                let inst = self.instance(key);
                let sig = self.instances[inst.0 as usize].sig.clone();
                let ty = Ty::Fn(Box::new(FnTy { params: sig.params, ret: sig.ret, fails: sig.fails, owning: false }));
                Ok(Expr { kind: K::FnRef(inst), ty, span })
            }
            Ok(ItemRef::ExternFn(_)) => err(Diagnostic::error(format!("`{}` is an extern function: it is called (under `no_check`), and is not a value to hold", path.spelled())).named("not_a_value").at(span)),
            Ok(ItemRef::Struct(_)) => err(Diagnostic::error(format!("`{}` is a struct: a value of it is built with `{}[field = …]`", path.spelled(), path.spelled())).named("not_a_value").at(span)),
            Ok(ItemRef::Enum(id)) => err(Diagnostic::error(format!("`{}` is an enum: a value is one of its cases — `{}.{}`", path.spelled(), path.spelled(), self.items.enums[id.0 as usize].cases.first().map(|c| c.name.as_str()).unwrap_or("case"))).named("not_a_value").at(span)),
            Ok(other) => err(Diagnostic::error(format!("`{}` is a {}, not a value", path.spelled(), other.what())).named("not_a_value").at(span)),
            Err(e) => {
                if path.segs.len() == 1 && path.kit.is_none() {
                    return err(Diagnostic::error(format!("there is no function `{}` here: a bare name is the function as a value, and none by that name is declared in {}", path.head(), crate::resolve::describe_module(self.items.module_path(ctx.module)))).named(e.name.unwrap_or("unknown_function")).at(span));
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
                    let cid = ctx.lookup_const(&n.name).or_else(|| self.items.modules[ctx.module.0 as usize].const_named(&n.name).map(|c| c.0));
                    // A const array read in place — an element, its length,
                    // lent to a call — is the program's own copy, nothing
                    // copied (owner, 2026-10-10).
                    if let (Some(cid), PlaceUse::Read | PlaceUse::Lend, false) = (cid, use_, ctx.is_task) {
                        let ty = self.const_type(cid, e.span)?;
                        if matches!(self.resolve(&ty).base(), Ty::Seq { kind: SeqKind::Array, .. }) {
                            let id = match ctx.const_views.iter().find(|(c, _)| *c == cid) {
                                Some((_, id)) => *id,
                                None => {
                                    let id = hir::LocalId(ctx.locals.len() as u32);
                                    ctx.locals.push(Local { name: format!(" const {}", n.name), ty: ty.clone(), mutability: None, kind: LocalKind::ConstView, from_blank: false, span: e.span });
                                    ctx.const_views.push((cid, id));
                                    id
                                }
                            };
                            return Ok(Place { base: id, proj: Vec::new(), ty, span: e.span });
                        }
                    }
                    if cid.is_some() {
                        return err(Diagnostic::error(format!("'{}' is a const and cannot be {}: it is a value known at compile time — nobody owns it, every use is a fresh value", n.name, match use_ { PlaceUse::Write => "assigned to", PlaceUse::Give | PlaceUse::Copy | PlaceUse::Lend | PlaceUse::LendMut => "given, copied or lent", PlaceUse::Read => "used as a place" })).named(if matches!(use_, PlaceUse::Write) { "const_changed" } else { "verb_on_const" }).at(e.span));
                    }
                    return self.undeclared(ctx, n);
                };
                self.check_place_use(ctx, id, use_, e.span, &n.name)?;
                if matches!(use_, PlaceUse::Write | PlaceUse::LendMut) {
                    ctx.changed.insert(id);
                }
                let local = &ctx.locals[id.0 as usize];
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
                    Ty::Enum { .. } => return err(Diagnostic::error(format!("`.{}` reaches into a struct; an enum's data is reached by `match`", field.name)).named("not_a_struct").at(field.span)),
                    other => return err(Diagnostic::error(format!("`.{}` reaches into a struct, and this is {}", field.name, crate::types::a_type(&self.spell(other)))).named("not_a_struct").at(field.span)),
                };
                let info = &self.items.structs[sid.0 as usize];
                let Some(fi) = info.fields.iter().position(|f| self.items.same_name(&self.items.case_fields, info.span, &f.name, &field.name)) else {
                    return err(Diagnostic::error(format!("`{}` has no field `{}`: its fields are {}", info.name, field.name, info.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", "))).named("unknown_field").at(field.span).fix_close(field.span, &field.name, info.fields.iter().map(|f| f.name.as_str())));
                };
                let f = &info.fields[fi];
                // A union holds one field at a time and only C knows which
                // (owner, 2026-10-19): reading a field is on your word.
                if info.union && !matches!(use_, PlaceUse::Write | PlaceUse::LendMut) && self.needs_mark(ctx) {
                    let whose = match &base.kind {
                        AK::Name(q) => format!("'{}'.{}", q.name, field.name),
                        _ => format!("`.{}`", field.name),
                    };
                    return err(Diagnostic::error(format!("reading {} is a `no_check` site: a `layout.C union` holds one field at a time, and only C knows which — write `no_check` in front of the statement", whose)).named("no_check_missing").at(e.span).fix_edit(e.span, Edit::Before, "no_check "));
                }
                if use_ == PlaceUse::Write && f.immut {
                    return err(Diagnostic::error(format!("`{}` is an `immut` field of `{}` and cannot be written", field.name, info.name)).named("cannot_change_immut").at(field.span));
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
                        return err(Diagnostic::error("one lane of a vector is read (`$v[1]`) and written (`change $v[3] = 30;`); lending one out to be changed is not decided yet").named("not_decided").at(e.span));
                    }
                    Ty::Simd { elem, .. } => (**elem).clone(),
                    Ty::Str(_) => return err(Diagnostic::error("a text has no `[…]`: it would not say which unit — the 3rd byte, point or grapheme? — use `std::str.at.<unit>` when std exists, or walk it with `str.each.<unit>`").named("text_unit_unsaid").at(e.span)),
                    Ty::Map { .. } => return err(Diagnostic::error("a map is looked up by key with `map.get(lend $m, key) otherwise …`, not by position").named("not_indexable").at(e.span)),
                    Ty::Bundle(_) => return err(Diagnostic::error("a mixed bundle has no `[…]`: walk it with `for`").named("not_indexable").at(e.span)),
                    other => {
                        let who = match &base.kind {
                            AK::Name(n) => format!("'{}'", n.name),
                            _ => "this".into(),
                        };
                        return err(Diagnostic::error(format!("{} is {}, neither a list nor a function: `$x(…)` reads a list, or calls a function held in 'x'", who, crate::types::a_type(&self.spell(other)))).named("not_a_function").at(e.span));
                    }
                };
                if use_ == PlaceUse::Give {
                    return err(Diagnostic::error("an element cannot be given away: it would leave a hole in the list — `list.remove(lend.mut $xs, i)` hands the element back").named("element_given_away").at(e.span));
                }
                let i = self.expr(ctx, index)?;
                let it = self.resolve(&i.ty);
                if !it.is_int() {
                    return err(Diagnostic::error(format!("a position is an integer, and this is {}", crate::types::a_type(&self.spell(&it)))).named("wrong_type").at(index.span));
                }
                p.proj.push(Proj::Index(Box::new(i)));
                p.ty = elem;
                p.span = e.span;
                Ok(p)
            }
            AK::Paren(inner) => self.place(ctx, inner, use_),
            AK::Verb { .. } => err(Diagnostic::error("one verb per value").named("two_verbs").at(e.span)),
            _ => match use_ {
                PlaceUse::Write => err(Diagnostic::error("only a variable, a field or an element can be assigned to").named("not_changeable").at(e.span)),
                PlaceUse::Give => err(Diagnostic::error("`give` takes a variable (or a field): only something you hold can be given away").named("verb_on_fresh_value").at(e.span)),
                PlaceUse::Copy => err(Diagnostic::error("`copy` takes a variable (or a field, or an element): a fresh value needs no copy").named("verb_on_fresh_value").at(e.span)),
                _ => err(Diagnostic::error("`lend` takes a variable (or a field, or an element): a fresh value goes straight in, with no verb").named("verb_on_fresh_value").at(e.span)),
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
        self.power_use(ctx, &["c_calls", "variables"], Some(info.name.clone()), format!("uses C's variable '{}'", info.name), span);
        if self.needs_mark(ctx) {
            self.power_first()?;
            let what = if use_ == PlaceUse::Write { "writing" } else { "reading" };
            return err(Diagnostic::error(format!("{} C's variable '{}' is a no_check site: nothing about C's memory can be proven — write `no_check` in front of the statement", what, info.name)).named("cnc::no_check_missing").at(span).fix_edit(span, Edit::Before, "no_check "));
        }
        if use_ == PlaceUse::Write && !info.mutable {
            return err(Diagnostic::error(format!("'{}' is read-only: `extern.C mut ${}: …` says C's variable may be written", info.name, info.name)).named("cnc::c_variable_read_only").at(span));
        }
        if matches!(use_, PlaceUse::Give | PlaceUse::Copy | PlaceUse::Lend | PlaceUse::LendMut) {
            return err(Diagnostic::error(format!("'{}' is C's plain value: it copies by itself and takes no verb", info.name)).named("nothing_to_give").at(span).fix_edit(span, Edit::Verb, ""));
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
                    return err(Diagnostic::error(format!("'{}' cannot be changed: it is `immut`", name)).named("cannot_change_immut").at(span).why_at(local.span, "declared `immut` here"));
                }
                match local.kind {
                    LocalKind::Lent => return err(Diagnostic::error(format!("'{}' is lent (shared) and cannot be changed: it belongs to whoever lent it — `lent.mut` would allow it", name)).named("changed_through_lend").at(span).why_at(local.span, "lent here")),
                    LocalKind::LoopVar | LocalKind::LoopLent => return err(Diagnostic::error(format!("'{}' belongs to the loop and cannot be changed: the header alone says what it is on every pass", name)).named(if local.kind == LocalKind::LoopLent { "changed_through_lend" } else { "counter_changed" }).at(span).why_at(local.span, "the loop's variable")),
                    _ => {}
                }
            }
            PlaceUse::LendMut => {
                if local.mutability == Some(ast::Mutability::Immut) {
                    return err(Diagnostic::error(format!("'{}' is `immut`: it cannot be lent mutably", name)).named("cannot_change_immut").at(span));
                }
                if local.kind == LocalKind::Lent {
                    return err(Diagnostic::error(format!("'{}' is lent (shared): it cannot be lent mutably on — `lent.mut` would allow it", name)).named("changed_through_lend").at(span));
                }
                if matches!(local.kind, LocalKind::LoopVar | LocalKind::LoopLent) {
                    return err(Diagnostic::error(format!("'{}' is the loop's element: it cannot be changed through the loop", name)).named(if local.kind == LocalKind::LoopLent { "changed_through_lend" } else { "counter_changed" }).at(span));
                }
            }
            PlaceUse::Give => {
                if matches!(local.kind, LocalKind::Lent | LocalKind::LentMut) {
                    return err(Diagnostic::error(format!("'{}' is lent: you cannot give away what you only borrowed", name)).named("lent_given_away").at(span).fix_edit(span, Edit::Verb, "copy"));
                }
                if matches!(local.kind, LocalKind::LoopVar | LocalKind::LoopLent) {
                    return err(Diagnostic::error(format!("'{}' is the loop's element and belongs to the list: `copy` it", name)).named("lent_given_away").at(span).fix_edit(span, Edit::Verb, "copy"));
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
                return err(Diagnostic::error(format!("`{}` on a fresh value: a call's result or a literal has no owner yet, so it goes straight in with no verb", verb.spelled())).named("verb_on_fresh_value").at(span).fix_edit(span, Edit::Verb, ""));
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
        err(Diagnostic::error(format!("`{}` on {} is not possible: {} is a plain value that owns nothing (no allocation behind it), so it copies by itself — write {} with no verb", verb.spelled(), name, crate::types::a_type(&self.spell(&ty)), name)).named("nothing_to_give").at(span))
    }

    /// A value about to be stored (into a `let`, a field, an element, a
    /// parameter): an owned value must arrive fresh or with its verb.
    pub fn check_stored_value(&self, ctx: &FnCtx, v: &Expr, target: &Ty) -> Result<()> {
        let t = self.resolve(target);
        let _ = ctx;
        // A number written into a C bit-field fits its bits (owner,
        // 2026-10-19); a written one is checked here, anything else where
        // it lands.
        if let Some(Bound::Int { lo, hi, said: crate::types::Said { bits: Some(n), .. } }) = t.bound() {
            let written = match &v.kind {
                K::Int(i) => Some(*i),
                K::Unary { op: UnOp::Neg, expr } => match &expr.kind {
                    K::Int(i) => Some(-*i),
                    _ => None,
                },
                _ => None,
            };
            if let Some(x) = written.filter(|x| x < lo || x > hi) {
                return err(Diagnostic::error(format!("{} does not fit a {}-bit field: `{}` holds {} to {}", x, n, self.spell(&t), lo, hi)).named("value_does_not_fit").at(v.span));
            }
        }
        if let K::Read(place) = &v.kind {
            let pt = self.resolve(&place.ty);
            if pt.is_owned() {
                let name = self.place_name(ctx, place);
                let fb = ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty();
                if !fb {
                    return err(Diagnostic::error(format!("{} is an owned `{}`: say what happens to it — `give {}` (it moves), `copy {}` (a second one), or `lend {}` (a view)", name, self.spell(&pt), name, name, name)).named("owned_value_bare").at(v.span).fix_edit(v.span, Edit::Verb, "give").fix_edit(v.span, Edit::Verb, "copy").fix_edit(v.span, Edit::Verb, "lend"));
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
    /// `5: u8` (owner, 2026-10-17): a literal, of the type after it,
    /// which it must fit, refused when building otherwise; never a name,
    /// which `to.T[…]` converts. A leading `-` is the literal's.
    fn typed_literal(&mut self, ctx: &mut FnCtx, value: &ast::Expr, ty: &ast::Type, span: Span) -> Result<Expr> {
        let t = self.body_type(ctx, ty)?;
        let tname = self.spell(&t);
        let (neg, lit) = match &value.kind {
            AK::Unary { op: ast::UnOp::Neg, expr } => (true, &**expr),
            _ => (false, value),
        };
        let (text, exact, whole) = match &lit.kind {
            AK::Int(x) => (x.replace('_', ""), false, true),
            AK::Bin(x) => (x.replace('_', ""), false, false),
            AK::Exact(x) => (x.replace('_', ""), true, !x.contains('.')),
            _ => {
                let what = match &value.kind {
                    AK::Name(q) => format!("'{}'", q.name),
                    _ => "…".to_string(),
                };
                return err(Diagnostic::error(format!("only a literal takes a type with `:` here (`5: u8`); convert a variable: `to.{}({})`", tname, what)).named("typed_literal_form").at(value.span));
            }
        };
        let written = format!("{}{}{}", if exact { "exactly " } else { "" }, if neg { "-" } else { "" }, text);
        let shown = format!("`{}: {}`", written, tname);
        match t.base() {
            Ty::Int(_) | Ty::Bin(_) if exact => err(Diagnostic::error(format!("{} is a deci: an `exactly` literal takes a deci type", shown)).named("wrong_type").at(span)),
            Ty::Int(_) if !whole => err(Diagnostic::error(format!("{} is not a whole number", shown)).named("wrong_type").at(span)),
            Ty::Int(_) => {
                let v = text.parse::<i128>().ok().map(|v| if neg { -v } else { v });
                let range = |ty: &Ty, items: &crate::resolve::Items| match crate::mir::domain::Domain::of_ty(ty, items) {
                    crate::mir::domain::Domain::Ints(set) => set.bounds(),
                    _ => None,
                };
                let base = range(t.base(), &self.items);
                let limits = range(&t, &self.items);
                let inside = |r: Option<(i128, i128)>| match (v, r) {
                    (Some(v), Some((lo, hi))) => lo <= v && v <= hi,
                    (_, None) => true,
                    (None, Some(_)) => false,
                };
                if !inside(base) {
                    let (lo, hi) = base.unwrap_or_default();
                    let bare = self.spell(t.base());
                    return err(Diagnostic::error(format!("{} does not fit: {} holds {} to {}", shown, bare, lo, hi)).named("value_does_not_fit").at(span));
                }
                if !inside(limits) {
                    return err(Diagnostic::error(format!("{} is outside the type's limits", shown)).named("value_does_not_fit").at(span));
                }
                Ok(Expr { kind: K::Int(v.unwrap_or_default()), ty: t, span })
            }
            Ty::Deci(_) if !exact && !whole => err(Diagnostic::error(format!("{} takes a deci from a bin literal, which is not decided: write `exactly {}: {}`", shown, written, tname)).named("money::exactly_needed").at(span).fix_edit(span, Edit::Insert, "exactly ")),
            Ty::Bin(_) | Ty::Deci(_) | Ty::Brain => {
                let v = self.expr(ctx, value)?;
                let mut v = self.expect_store(ctx, v, &t, &shown)?;
                v.ty = t;
                Ok(v)
            }
            _ => err(Diagnostic::error(format!("{} needs a number type", shown)).named("typed_literal_form").at(span)),
        }
    }

    pub fn expect_store(&mut self, ctx: &mut FnCtx, v: Expr, target: &Ty, what: &str) -> Result<Expr> {
        let t = self.resolve(target);
        let vt = self.resolve(&v.ty);
        // The places a `d` or a percent keeps (owner, 2026-10-17): a literal
        // with more is refused, and a value that could have more is
        // rounded on purpose.
        if let Some(keep) = places_max(&t) {
            let spelled = self.spell(&t);
            match self.places_in(&v) {
                PlacesOf::Literal(n) if n > keep => return err(Diagnostic::error(format!("this literal has {} places; {} keeps {}: write fewer, or round it on purpose", n, spelled, keep)).named("money::too_many_places").at(v.span)),
                PlacesOf::Can(n) if n > keep => return err(Diagnostic::error(format!("this can have {} places; {} keeps {}: round it on purpose, `round.near(…, places: {})`", n, spelled, keep, keep)).named("money::too_many_places").at(v.span)),
                PlacesOf::Unknown => return err(Diagnostic::error(format!("nothing says how many places this has; {} keeps {}: round it on purpose, `round.near(…, places: {})`", spelled, keep, keep)).named("money::too_many_places").at(v.span)),
                _ => {}
            }
        }
        // A percent and a plain number are two families (owner,
        // 2026-10-17): each becomes the other only through `to.`.
        match (percent_inner(&vt), percent_inner(&t)) {
            (Some(_), None) if matches!(t.base(), Ty::Bin(_) | Ty::Deci(_) | Ty::Int(_) | Ty::Brain) => {
                let to = self.spell(&t.base().clone());
                return err(Diagnostic::error(format!("this is a percent, not {to}; a percent becomes a plain number with `to.{to}(…)` (7% → 0.07)")).named("money::percent_mixed").at(v.span));
            }
            (None, Some(inner)) if matches!(vt.base(), Ty::Bin(_) | Ty::Deci(_) | Ty::Int(_) | Ty::Brain) => {
                let pc = self.spell(&inner);
                return err(Diagnostic::error(format!("this is not a percent; a plain number becomes one with `to.percent[{pc}](…)` (0.07 → 7%)")).named("money::percent_mixed").at(v.span));
            }
            _ => {}
        }
        // `simd.fill` into a declared vector: the declared lanes win, and
        // the value is fitted into one as a typed `let` fits it (owner,
        // 2026-10-19: "yea" to (a)). `simd.fill[value 5: u8]` into a
        // `simd[i32, count 4]` is four `i32` lanes of 5.
        let fill_lane = match (t.base(), vt.base(), &v.kind) {
            (Ty::Simd { elem: lane, .. }, Ty::Simd { elem: had, .. }, K::Call { callee: hir::Callee::Builtin(hir::Builtin::SimdFill), args, .. })
                if matches!(args.first(), Some(hir::Arg::Value(_))) && self.resolve(lane) != self.resolve(had) && !matches!(self.resolve(had).base(), Ty::Var(_)) =>
            {
                Some((**lane).clone())
            }
            _ => None,
        };
        if let Some(lane) = fill_lane {
            let mut v = v;
            let span = v.span;
            if let K::Call { args, .. } = &mut v.kind {
                if let Some(hir::Arg::Value(inner)) = args.first_mut() {
                    let taken = std::mem::replace(inner, Expr { kind: K::Bool(false), ty: Ty::Bool, span });
                    *inner = self.expect_store(ctx, taken, &lane, "every lane")?;
                }
            }
            v.ty = t.base().clone();
            return Ok(v);
        }
        // A bare integer literal takes its context's family; a *name* never
        // does (owner, 2026-09-22): Precog's freedom is over representation —
        // width, list or array, map order, which checks survive — never over
        // meaning, and 70000 in an `f16` would silently become 69984.
        // Bare numbers in a `deci` place are decimals, exactly as written
        // (owner, 2026-09-29): `let 'a': d64 = 0.1 + 0.2;` is 0.3.
        if matches!(vt.base(), Ty::Deci(EXACT_FREE)) && matches!(t.base(), Ty::Bin(_) | Ty::Int(_)) {
            return err(Diagnostic::error(format!("a `*` literal is exactly what is written, a deci, and {} is {}, which cannot hold it exactly: drop the `*` for a bin, or make it a `deci`", what, article(&self.spell(&t)))).named("money::exact_into_bin").at(v.span));
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
                return err(Diagnostic::error("this text is not ASCII, so it cannot be str.ascii").named("text_not_ascii").at(at));
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
                return err(Diagnostic::error(format!("`{}` is not one {}: it is {} {}, so it fits `char.{}`, not `{}`", c, match u { Unit::Byte => "byte", Unit::Point => "code point", _ => "grapheme" }, n, what, match need { Unit::Byte => "byte", Unit::Point => "point", _ => "grapheme" }, self.spell(&t))).named("not_one_code_point").at(v.span));
            }
        }
        // Arithmetic made only of literals has no family either, so in a
        // typed place it takes the place's: `'m': u64 =
        // 9223372036854775807 + 1` is a `u64` sum, 2^63, not an
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
                // 2026-09-23): `let 'tiny': i8 = (100 + 100) - 150;` is
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
            // A list literal is built as the place's list: a struct's
            // field or an argument holds `list[i32]`, never the bare
            // numbers' own 64-bit `list[integer]` (examples/gradebook: Bo's
            // `[40, 55]` was read as 40, 0 once appended to).
            let mut v = v;
            if matches!(v.kind, K::List(_)) && matches!(t.base(), Ty::Seq { .. }) {
                v.ty = t.base().clone();
            }
            return Ok(v);
        }
        // Say it in the writer's terms.
        let msg = match (vt.base(), t.base()) {
            (Ty::Brain, other) | (other, Ty::Brain) if other.is_numeric() => {
                let (have, want) = (self.spell(&vt), self.spell(&t));
                format!("{} is {} and this value is a {}, not {}: the brain float is a family of its own — convert it, `to.{}(…)`", what, article(&want), have, want, conv_spelling(&want))
            }
            (Ty::Bin(_) | Ty::Deci(_), Ty::Int(_)) => {
                format!("{} is {} and this value is {}: there is no implicit conversion between numeric families — write `to.{}(…) otherwise …`, which fails `not_whole` rather than dropping the fraction", what, article(&self.spell(&t)), article(&self.spell(&vt)), conv_spelling(&self.spell(t.base())))
            }
            (Ty::Int(_), Ty::Bin(_)) | (Ty::Int(_), Ty::Deci(_)) | (Ty::Bin(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Bin(_)) => {
                format!("{} is {} and this value is {}: there is no implicit conversion between numeric families — write `to.{}(…)`", what, article(&self.spell(&t)), article(&self.spell(&vt)), conv_spelling(&self.spell(t.base())))
            }
            (Ty::Str(_), Ty::Int(_) | Ty::Bin(_)) => format!("{} is {} and this value is text: convert it first — `to.{}(…) otherwise …` (it can fail with `not_a_number`)", what, article(&self.spell(&t)), conv_spelling(&self.spell(t.base()))),
            (Ty::Int(_) | Ty::Bin(_), Ty::Str(_)) => format!("{} is text and this value is {}: convert it first with `to.str.utf8(…)`", what, crate::types::a_type(&self.spell(&vt))),
            (Ty::Seq { .. }, Ty::Int(_)) if matches!(v.kind, K::List(ref l) if l.is_empty()) => format!("`[]` is an empty list, but {} is {}", what, crate::types::a_type(&self.spell(&t))),
            (Ty::Seq { kind: SeqKind::List, .. }, Ty::Seq { kind: SeqKind::Array, .. }) => format!("{} is an `array` and this is a `list`", what),
            // Two types spelled alike are two Kits' (owner, 2026-09-30: one
            // Kit under two prefixes): each named with its own.
            _ if self.spell(&t) == self.spell(&vt) => format!("expected {} ({}), found {}", what.replace("the ", "a value of the "), self.spell_with_kit(&t), article(&self.spell_with_kit(&vt))),
            _ => format!("expected {} ({}), found {}", what.replace("the ", "a value of the "), self.spell(&t), article(&self.spell(&vt))),
        };
        // A conversion that cannot fail, around the value: `to.f64($n)`.
        let convert = match (vt.base(), t.base()) {
            (Ty::Int(_), Ty::Bin(_) | Ty::Deci(_)) | (Ty::Bin(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Bin(_)) => Some(format!("to.{}", conv_spelling(&self.spell(t.base())))),
            (Ty::Brain, other) | (other, Ty::Brain) if other.is_numeric() => Some(format!("to.{}", conv_spelling(&self.spell(t.base())))),
            (Ty::Int(_) | Ty::Bin(_), Ty::Str(_)) => Some("to.str.utf8".to_string()),
            _ => None,
        };
        let msg = Diagnostic::error(msg);
        let msg = match convert {
            Some(c) => msg.fix_edit(v.span, Edit::Wrap, c),
            None => msg,
        };
        err(msg.named(match (vt.base(), t.base()) { (Ty::Brain, other) | (other, Ty::Brain) if other.is_numeric() => "ai::brain_mixed", (Ty::Bin(_) | Ty::Deci(_), Ty::Int(_)) | (Ty::Int(_), Ty::Bin(_)) | (Ty::Int(_), Ty::Deci(_)) | (Ty::Bin(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Bin(_)) => "no_implicit_conversion", (Ty::Seq { kind: SeqKind::List, .. }, Ty::Seq { kind: SeqKind::Array, .. }) => "list_array_mismatch", (Ty::Fn(_), Ty::Fn(_)) => "function_shape_wrong", (Ty::Struct { id: a, .. }, Ty::Struct { id: b, .. }) if a == b => "generic_filled_inconsistently", (Ty::Enum { id: a, .. }, Ty::Enum { id: b, .. }) if a == b => "generic_filled_inconsistently", _ => "wrong_type" }).at(v.span))
    }

    // ------------------------------------------------------------ operators

    /// Only a bin or brain16, or a vector of them, has a -0 for `.ieee` to
    /// keep (owner, 2026-10-07).
    fn ieee_takes(&self, spelled: &str, plain: &str, ty: &Ty, at: Span) -> Result<()> {
        let lane = match ty.base() {
            Ty::Simd { elem, .. } => elem.base().clone(),
            t => t.clone(),
        };
        if matches!(lane, Ty::Bin(_) | Ty::Brain) {
            return Ok(());
        }
        let none = match lane {
            Ty::Int(_) => ", and an integer has none".to_string(),
            Ty::Deci(_) => ", and a deci has none".to_string(),
            _ => String::new(),
        };
        err(Diagnostic::error(format!("`{}` keeps a -0, which only a bin or brain16 has{}: write `{}`", spelled, none, plain)).named("ieee_misused").at(at))
    }

    fn unary(&mut self, ctx: &mut FnCtx, op: UnOp, operand: &ast::Expr, span: Span) -> Result<Expr> {
        let v = self.expr(ctx, operand)?;
        let vt = self.resolve(&v.ty);
        // A percent negates to a percent (owner: `-5%`, `-'tax'`).
        if matches!(op, UnOp::Neg | UnOp::NegIeee) && percent_inner(&vt).is_some() {
            let mut inner = v;
            inner.ty = percent_inner(&vt).unwrap();
            let mut e = self.unary_on(op, inner, span)?;
            e.ty = Ty::Percent(Box::new(e.ty.clone()));
            return Ok(e);
        }
        self.unary_on(op, v, span)
    }

    /// A unary operator on its operand, checked already.
    fn unary_on(&mut self, op: UnOp, v: Expr, span: Span) -> Result<Expr> {
        let vt = self.resolve(&v.ty);
        match op {
            // `negate.ieee 'x'` (owner, 2026-10-07: "yea").
            UnOp::NegIeee => {
                self.ieee_takes("negate.ieee", "-'x'", &vt, span)?;
                let ty = vt.base().clone();
                Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span })
            }
            UnOp::Neg => {
                if !vt.is_numeric() {
                    return err(Diagnostic::error(format!("`-` negates a number, and this is {}", crate::types::a_type(&self.spell(&vt)))).named("operator_wrong_type").at(span));
                }
                if let Ty::Int(IntW::Fixed { signed: false, .. }) = vt.base() {
                    return err(Diagnostic::error(format!("`-` on {}: an unsigned number is never negative", crate::types::a_type(&self.spell(&vt)))).named("negative_unsigned").at(span));
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
                // ('v' + 1)` with 'v' a `u32` flips 32 bits
                // (`widths.cpls`), not 65.
                if vt.is_exact() {
                    let ty = self.size_of_values(&v).ok_or_else(|| Box::new(Diagnostic::error("not yet: `not.bit` on an exact answer whose operands have different sizes (or none left to see) — which size it flips is open").named("not_decided").at(span)))?;
                    return Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span });
                }
                match vt.base() {
                    Ty::Int(IntW::Fixed { .. }) => {}
                    Ty::Int(IntW::Free(_)) if literal_only(&v) => {
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), span, "not.bit".to_string()));
                        return Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty: t, span });
                    }
                    Ty::Int(IntW::Free(_)) => return err(Diagnostic::error("`not.bit` only knows a declared width: it flips every bit, so the value needs a type (`let $x: u8 = …`)").named("width_unknown").at(span)),
                    other => return err(Diagnostic::error(format!("`not.bit` works on integers, and this is {}", crate::types::a_type(&self.spell(other)))).named("needs_integer").at(span)),
                }
                let ty = vt.base().clone();
                Ok(Expr { kind: K::Unary { op, expr: Box::new(v) }, ty, span })
            }
        }
    }

    fn binary(&mut self, ctx: &mut FnCtx, op: &BinOp, op_span: Span, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        // `plus.ieee`, `minus.ieee`, `times.ieee` (owner, 2026-10-07):
        // checked as the plain operator, then kept as IEEE's own answer.
        if matches!(op, BinOp::AddIeee | BinOp::SubIeee | BinOp::MulIeee | BinOp::ModIeee(_) | BinOp::PowIeee) {
            let mut e = self.binary(ctx, &op.plain(), op_span, lhs, rhs, span)?;
            let ty = self.resolve(&e.ty);
            self.ieee_takes(&op.spelled(), &op.plain().spelled(), &ty, op_span)?;
            if let K::Binary { op: o, .. } = &mut e.kind {
                *o = op.clone();
            }
            return Ok(e);
        }
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
            return err(Diagnostic::error(format!("`{}` is not defined for `{}`: declare `op {} [$a: {}, $b: {}] -> …` to give it one", op.spelled(), self.spell(&lt), op.spelled(), self.spell(&lt), self.spell(&lt))).named("unknown_operator").at(op_span));
        }
        let r = self.expr(ctx, rhs)?;
        let rt = self.resolve(&r.ty);
        // An `exactly` literal is left unsettled here: beside a deci it
        // takes that deci's width (`'p' + exactly 7` with 'p' a d32 is a
        // d32, as the oracle reads it), which `binary_checked` decides; it
        // was made a d64 first, and the two widths were refused.
        let l = self.operand_unsettled(ctx, l)?;
        let r = self.operand_unsettled(ctx, r)?;
        self.binary_checked(ctx, op, op_span, l, r, lt, rt, lhs, rhs, span)
    }

    /// An operator with a percent on one side or both (owner, 2026-10-17):
    /// percents add, subtract and compare among themselves, a `%` literal
    /// beside a plain deci is the number over 100, and nothing else mixes.
    #[allow(clippy::too_many_arguments)]
    fn percent_binary(&mut self, ctx: &mut FnCtx, op: &BinOp, op_span: Span, mut l: Expr, mut r: Expr, mut lt: Ty, mut rt: Ty, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        let literal = |e: &ast::Expr| matches!(e.kind, AK::Percent(_));
        // A `%` literal beside a plain number is its fraction (owner,
        // 2026-10-16: "a plain 80% is a bin"): `'p' + exactly 7%` is
        // 'p' + 0.07, `'x' + 80%` is 'x' + 0.8.
        if percent_inner(&rt).is_none() && literal(lhs) && plain_number(&rt) {
            lt = percent_inner(&self.resolve(&l.ty)).unwrap();
            l.ty = lt.clone();
            return self.binary_checked(ctx, op, op_span, l, r, lt, rt, lhs, rhs, span);
        }
        if percent_inner(&lt).is_none() && literal(rhs) && plain_number(&lt) {
            rt = percent_inner(&self.resolve(&r.ty)).unwrap();
            r.ty = rt.clone();
            return self.binary_checked(ctx, op, op_span, l, r, lt, rt, lhs, rhs, span);
        }
        let (li, ri) = (percent_inner(&lt), percent_inner(&rt));
        let (Some(li), Some(ri)) = (li.clone(), ri.clone()) else {
            let other = if li.is_some() { &rt } else { &lt };
            let fam = match other.base() {
                Ty::Bin(_) => "a bin",
                Ty::Deci(_) => "a deci",
                Ty::Int(_) => "a whole number",
                Ty::Brain => "a brain16",
                _ => return err(Diagnostic::error(format!("`{}` between a percent and {}: a percent is a number", op.spelled(), crate::types::a_type(&self.spell(other)))).named("operator_wrong_type").at(op_span)),
            };
            return err(Diagnostic::error(format!("`{}` between a percent and {}: a percent is its own number family; convert it, `to.d64($tax)` (7% → 0.07), or the number, `to.percent[d64](0.07)`", op.spelled(), fam)).named("money::percent_mixed").at(op_span));
        };
        if !(matches!(op, BinOp::Add | BinOp::Sub) || op.is_comparison()) {
            let name = |e: &ast::Expr| match &e.kind {
                AK::Name(q) => format!("'{}'", q.name),
                _ => "…".to_string(),
            };
            let (a, b) = (name(lhs), name(rhs));
            let b = if a == b { "'b'".to_string() } else { b };
            return err(Diagnostic::error(format!("`{}` between two percents is not decided; convert them: `to.d64({}) {} to.d64({})`", op.spelled(), a, match op { BinOp::Mul => "x".to_string(), BinOp::Pow => "xx".to_string(), o => o.spelled() }, b)).named("not_decided").at(op_span));
        }
        // Worked out on their numbers; a sum or difference is a percent.
        let (li, ri) = (percent_inner(&self.resolve(&l.ty)).unwrap_or(li), percent_inner(&self.resolve(&r.ty)).unwrap_or(ri));
        l.ty = li.clone();
        r.ty = ri.clone();
        let mut e = self.binary_checked(ctx, op, op_span, l, r, li, ri, lhs, rhs, span)?;
        if matches!(op, BinOp::Add | BinOp::Sub) {
            e.ty = Ty::Percent(Box::new(e.ty.clone()));
        }
        Ok(e)
    }

    /// A binary operator on its two operands, checked already.
    #[allow(clippy::too_many_arguments)]
    fn binary_checked(&mut self, ctx: &mut FnCtx, op: &BinOp, op_span: Span, mut l: Expr, mut r: Expr, lt: Ty, mut rt: Ty, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        // A percent is its own number family (owner, 2026-10-17).
        if percent_inner(&lt).is_some() || percent_inner(&rt).is_some() {
            return self.percent_binary(ctx, op, op_span, l, r, lt, rt, lhs, rhs, span);
        }
        // A comparison compares true values (owner, 2026-09-23), so a group
        // of bare numbers on either side is worked out exactly — not held at
        // `i64` beside another bare number (seed 439), nor squeezed into
        // the other side's width.
        let mut lt = lt;
        if op.is_comparison() && lt.is_int() && rt.is_int() {
            // At the other side's width, or the bare numbers' own size,
            // at least 64 bits (owner, 2026-10-03): `'m' == -2^126` beside
            // an `i128`.
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
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_) => {
                if lt.base() == &Ty::Duration || rt.base() == &Ty::Duration {
                    if !(lt.base() == &Ty::Duration && rt.base() == &Ty::Duration) || !matches!(op, BinOp::Add | BinOp::Sub) {
                        if matches!(op, BinOp::Mul) && (lt.is_int() || rt.is_int()) {
                            return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Duration, span });
                        }
                        return err(Diagnostic::error(format!("`{}` between a duration and {} has no meaning: durations add, subtract, compare, and multiply by a whole number", op.spelled(), crate::types::a_type(&self.spell(if lt.base() == &Ty::Duration { &rt } else { &lt })))).named("operator_wrong_type").at(op_span));
                    }
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Duration, span });
                }
                let (l, r, ty) = self.numeric_pair(ctx, l, r, op, op_span)?;
                // Only a bin or brain16 has a -0 to keep (owner, 2026-10-07).
                if let BinOp::DivIeee(_) = op {
                    self.ieee_takes(&op.spelled(), &op.plain().spelled(), &ty, op_span)?;
                }
                self.rounding_takes(op, op_span, &ty)?;
                if *op == BinOp::Pow && ty.is_int() {
                    if let K::Int(e) = &r.kind {
                        if *e < 0 {
                            return err(Diagnostic::error("a negative exponent on integers would produce a fraction: `xx` on integers takes a non-negative power").named("negative_exponent").at(rhs.span));
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
                // A shift still waiting for its width shares this one's.
                if self.width_pending(&lt) || self.width_pending(&rt) {
                    let (l, r, ty) = self.numeric_pair(ctx, l, r, op, op_span)?;
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
                        let mut l = l;
                        exact_group(&mut l);
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                    }
                    Ty::Int(IntW::Free(_)) if literal_only(&l) && literal_only(&r) => {
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), op_span, op.spelled().to_string()));
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                    }
                    Ty::Int(IntW::Free(_)) => return err(Diagnostic::error(format!("`{}` wraps at the type's size, so it only knows a declared one: give the left operand a type (`let $x: u32 = …`)", op.spelled())).named("width_unknown").at(op_span)),
                    other => return err(Diagnostic::error(format!("`{}` wraps an integer, and this is {}: it only knows a declared integer width", op.spelled(), crate::types::a_type(&self.spell(other)))).named("needs_integer").at(op_span)),
                }
                if !rt.is_int() {
                    return err(Diagnostic::error(format!("`{}` takes integers on both sides; the right side is {}", op.spelled(), crate::types::a_type(&self.spell(&rt)))).named("needs_integer").at(rhs.span));
                }
                let ty = lt.base().clone();
                // A group of bare numbers beside the typed side is plain
                // maths, as wide as its own values need, landing in the
                // operator's width (check claude, seed 34247: `'v'
                // plus.wrap (9223372036854775807 plus 1)` with a `u64`).
                let mut r = r;
                exact_group(&mut r);
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
            }
            BinOp::Join => {
                if !lt.is_text() {
                    if lt.is_char() && rt.is_text() || lt.is_char() && rt.is_char() {
                        // a char joins as text
                    } else if lt.is_numeric() {
                        return err(Diagnostic::error(format!("`concat` joins text, and its left side is {}: convert it first — `to.str.utf8(…) concat …`", article(&self.spell(&lt)))).named("concat_needs_text").at(lhs.span));
                    } else {
                        return err(Diagnostic::error(format!("`concat` joins text, and its left side is {}", crate::types::a_type(&self.spell(&lt)))).named("concat_needs_text").at(lhs.span));
                    }
                }
                if !rt.is_text() && !rt.is_char() {
                    if rt.is_numeric() {
                        return err(Diagnostic::error(format!("`concat` joins text, and its right side is {}: convert it first — `… concat to.str.utf8(…)`", article(&self.spell(&rt)))).named("concat_needs_text").at(rhs.span));
                    }
                    return err(Diagnostic::error(format!("`concat` joins text, and its right side is {}", crate::types::a_type(&self.spell(&rt)))).named("concat_needs_text").at(rhs.span));
                }
                let enc = match (lt.base(), rt.base()) {
                    (Ty::Str(e), _) if !matches!(e, Enc::Free(_)) => *e,
                    (_, Ty::Str(e)) => *e,
                    _ => Enc::Free(InferId(0)),
                };
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Str(enc), span })
            }
            BinOp::Eq | BinOp::Ne => {
                // A comparison compares true values: a group of bare numbers
                // on either side is worked exactly, as wide as it needs.
                let (mut l, mut r) = (l, r);
                self.bare_group_exact(&mut l);
                self.bare_group_exact(&mut r);
                let lt = self.resolve(&l.ty);
                let r = self.expect_operand(ctx, r, &lt, op, op_span)?;
                if matches!(lt.base(), Ty::Fn(_) | Ty::Ptr { .. } | Ty::Channel(_)) {
                    return err(Diagnostic::error(format!("`equal_to` has no meaning for {}", crate::types::a_type(&self.spell(&lt)))).named("operator_wrong_type").at(op_span));
                }
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                // A shift still waiting for its width meets a comparison,
                // which holds the answer, never its operands: they take
                // their width from each other (`operators.md`).
                let pending = |this: &Self, t: &Ty| matches!(t, Ty::Infer(_)) && this.widths_pending.iter().any(|(p, _, _)| this.resolve(p) == *t);
                if pending(self, &lt) || pending(self, &rt) {
                    let (l, r, _) = self.numeric_pair(ctx, l, r, op, op_span)?;
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span });
                }
                // A comparison compares true values: a group of bare numbers
                // on either side is worked exactly, as wide as it needs.
                let (mut l, mut r) = (l, r);
                self.bare_group_exact(&mut l);
                self.bare_group_exact(&mut r);
                let lt = self.resolve(&l.ty);
                if lt.is_text() {
                    return err(Diagnostic::error(format!("`{}` on text has no meaning yet: byte order is never what anyone means, and a collation rule is std's — compare lengths or use a std function", op.spelled())).named("text_order_undecided").at(op_span));
                }
                // Characters compare by content (`==`); an order between
                // them was never decided, and the oracle refuses one.
                if lt.is_char() {
                    return err(Diagnostic::error(format!("`{}` on characters has no meaning yet: `equal_to` compares them, and `char.code(…)` gives the number under one", op.spelled())).named("text_order_undecided").at(op_span));
                }
                if !lt.is_numeric() && lt.base() != &Ty::Duration {
                    return err(Diagnostic::error(format!("`{}` compares numbers, and this is {}", op.spelled(), crate::types::a_type(&self.spell(&lt)))).named("operator_wrong_type").at(op_span));
                }
                let r = self.expect_operand(ctx, r, &lt, op, op_span)?;
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::And | BinOp::Or | BinOp::AndThen | BinOp::OrElse => {
                self.expect_bool(&l, &format!("the left side of `{}`", op.spelled()))?;
                self.expect_bool(&r, &format!("the right side of `{}`", op.spelled()))?;
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: Ty::Bool, span })
            }
            BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::AndBitWrap | BinOp::OrBitWrap | BinOp::XorWrap | BinOp::Shl | BinOp::Shr | BinOp::ShlMasked | BinOp::ShrMasked => {
                let (mut l, mut r) = (l, r);
                // A count is a count, exact or not; a group beside an
                // exact answer is worked exactly too.
                if op.is_shift() {
                    self.bare_group_exact(&mut r);
                } else if lt.is_exact() {
                    self.bare_group_exact(&mut r);
                } else if rt.is_exact() {
                    self.bare_group_exact(&mut l);
                }
                let (lt, rt) = (self.resolve(&l.ty), self.resolve(&r.ty));
                if !lt.is_int() || !rt.is_int() {
                    let (bad, side) = if !lt.is_int() { (&lt, &l) } else { (&rt, &r) };
                    // A shift whose width is not known yet is a widthless
                    // shift: a count gives nothing inside it a width (owner,
                    // 2026-10-17: "(a)", check claude's `to.i64[1] shl (5
                    // shr to.u8[1])`).
                    if bad.is_infer() {
                        if let K::Binary { op: inner, op_span: at, .. } = &side.kind {
                            if inner.is_shift() {
                                return err(Diagnostic::error(format!("`{}` needs a width, and nothing here gives one: declare where the value goes (`let $v: u32 = 1 {} 3;`) or write `to.u32(1) {} 3`", inner.spelled(), inner.spelled(), inner.spelled())).named("width_unknown").at(*at));
                            }
                        }
                    }
                    return err(Diagnostic::error(format!("`{}` works on integers, and this is {}", op.spelled(), crate::types::a_type(&self.spell(bad)))).named("needs_integer").at(op_span));
                }
                // A shift works at the width of what it shifts: its count is
                // a count, exact or not. On an exact answer, the width of the
                // typed operands inside it, as `plus.wrap` takes it (owner,
                // 2026-09-23, confirmed 2026-10-03).
                if matches!(op, BinOp::Shl | BinOp::Shr | BinOp::ShlMasked | BinOp::ShrMasked) && (lt.is_exact() || rt.is_exact()) {
                    let ty = if lt.is_exact() {
                        self.size_of_values(&l).ok_or_else(|| self.open_size(op, op_span))?
                    } else if matches!(lt.base(), Ty::Int(IntW::Fixed { .. })) {
                        lt.base().clone()
                    } else {
                        // A bare number shifted takes its width from where
                        // the value goes, never from its count (owner,
                        // 2026-10-07: "yea"; as ARM's `lsl` shifts at the
                        // shifted register's width).
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), op_span, op.spelled().to_string()));
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                    };
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                }
                if lt.is_exact() || rt.is_exact() {
                    let ty = match (lt.is_exact(), rt.is_exact()) {
                        // A group of bare numbers has no typed operands: the
                        // other side's width decides.
                        (true, true) if literal_only(&r) => self.size_of_values(&l).ok_or_else(|| self.open_size(op, op_span))?,
                        (true, true) if literal_only(&l) => self.size_of_values(&r).ok_or_else(|| self.open_size(op, op_span))?,
                        (true, true) => self.size_of_sides(&[&l, &r]).ok_or_else(|| self.open_size(op, op_span))?,
                        (true, false) if matches!(rt.base(), Ty::Int(IntW::Fixed { .. })) => rt.base().clone(),
                        (false, true) if matches!(lt.base(), Ty::Int(IntW::Fixed { .. })) => lt.base().clone(),
                        (true, false) => self.size_of_values(&l).ok_or_else(|| self.open_size(op, op_span))?,
                        _ => self.size_of_values(&r).ok_or_else(|| self.open_size(op, op_span))?,
                    };
                    return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span });
                }
                // A shift's width is the shifted number's own type, or, when
                // it is bare, where the value goes; the count never decides
                // it (owner, 2026-10-07: "yea").
                if matches!(op, BinOp::Shl | BinOp::Shr | BinOp::ShlMasked | BinOp::ShrMasked) && matches!(lt.base(), Ty::Int(IntW::Free(_))) {
                    if literal_only(&l) {
                        let t = self.fresh();
                        self.widths_pending.push((t.clone(), op_span, op.spelled().to_string()));
                        return Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty: t, span });
                    }
                    return err(Diagnostic::error(format!("`{}` only knows a declared width: shifting can lose bits, so the shifted number needs a type", op.spelled())).named("width_unknown").at(op_span));
                }
                let ty = if matches!(lt.base(), Ty::Int(IntW::Fixed { .. })) { lt.base().clone() } else { rt.base().clone() };
                // A group of bare numbers beside a typed operand is plain
                // maths: worked exactly, as wide as its own values need,
                // then landing in the operator's width (check claude, seeds
                // 25263 and 26090: `'v' xor (15803055715966303544 minus 8)`
                // with a `u64` 'v' is one 64-bit pattern, not past
                // `i64`).
                let (mut l, mut r) = (l, r);
                if matches!(ty, Ty::Int(IntW::Fixed { .. })) {
                    for side in [&mut l, &mut r] {
                        if literal_only(side) && !matches!(side.kind, K::Int(_)) && matches!(self.resolve(&side.ty).base(), Ty::Int(IntW::Free(_))) {
                            if let Some(bits) = literal_group_bits(side) {
                                retype_literal_arith(side, &Ty::exact_at(bits.max(64)));
                            }
                        }
                    }
                }
                Ok(Expr { kind: K::Binary { op: op.clone(), lhs: Box::new(l), rhs: Box::new(r), op_span }, ty, span })
            }
            BinOp::Custom(_) | BinOp::AddIeee | BinOp::SubIeee | BinOp::MulIeee | BinOp::ModIeee(_) | BinOp::PowIeee => unreachable!(),
        }
    }

    /// "Wrapping wraps at the size of its values" (`operators.md`): for an
    /// exact sum, the one type its typed operands share. `None` when they
    /// differ, or when there are none left to see (a `:=` name holding an
    /// exact answer) — which size then is open (asked 2026-09-23).
    fn size_of_values(&self, e: &Expr) -> Option<Ty> {
        self.size_of_sides(&[e])
    }

    /// The one size the typed operands of all these share; with none, the
    /// one their `simd.sum`s' lanes share.
    fn size_of_sides(&self, sides: &[&Expr]) -> Option<Ty> {
        let (mut typed, mut lanes) = (Vec::new(), Vec::new());
        for e in sides {
            let (t, l) = self.sizes_and_lanes(e)?;
            typed.extend(t);
            lanes.extend(l);
        }
        let one = |v: Vec<Ty>| -> Option<Ty> {
            let first = v.first()?.clone();
            v.iter().all(|t| *t == first).then_some(first)
        };
        if typed.is_empty() { one(lanes) } else { one(typed) }
    }

    /// Every size among an exact answer's typed operands; `None` when one
    /// has no size to see.
    fn sizes_of_values(&self, e: &Expr) -> Option<Vec<Ty>> {
        let (typed, lanes) = self.sizes_and_lanes(e)?;
        let mut out: Vec<Ty> = Vec::new();
        for s in if typed.is_empty() { lanes } else { typed } {
            if !out.contains(&s) {
                out.push(s);
            }
        }
        Some(out)
    }

    /// The sizes of an exact answer's typed operands, and apart from them
    /// the lanes of each `simd.sum` in it: a plain number (owner,
    /// 2026-10-07: "b") that gives way to a typed operand and otherwise
    /// works at its lane's type, as the oracle does (check claude, seed
    /// 32250).
    fn sizes_and_lanes(&self, e: &Expr) -> Option<(Vec<Ty>, Vec<Ty>)> {
        let t = self.resolve(&e.ty);
        if !t.is_exact() {
            return match t.base() {
                Ty::Int(IntW::Fixed { .. }) => Some((vec![t.base().clone()], Vec::new())),
                _ => None,
            };
        }
        let mut sizes: Vec<Ty> = Vec::new();
        let mut lanes: Vec<Ty> = Vec::new();
        fn walk(this: &Checker, e: &Expr, sizes: &mut Vec<Ty>, lanes: &mut Vec<Ty>) -> bool {
            let t = this.resolve(&e.ty);
            if literal_only(e) {
                return true;
            }
            if let K::Call { callee: hir::Callee::Builtin(hir::Builtin::SimdSum), args, .. } = &e.kind {
                if let Some(Arg::Value(v)) = args.first() {
                    if let Ty::Simd { elem, .. } = this.resolve(&v.ty).base() {
                        lanes.push(elem.base().clone());
                        return true;
                    }
                }
            }
            if !t.is_exact() {
                if let Ty::Int(IntW::Fixed { .. }) = t.base() {
                    sizes.push(t.base().clone());
                    return true;
                }
                return false;
            }
            match &e.kind {
                K::Binary { lhs, rhs, .. } => walk(this, lhs, sizes, lanes) && walk(this, rhs, sizes, lanes),
                // A negative keeps its operand's size (check claude:
                // `(-('a' minus 1)) shr 1` with an `i8` 'a').
                K::Unary { expr, .. } => walk(this, expr, sizes, lanes),
                _ => false,
            }
        }
        if !walk(self, e, &mut sizes, &mut lanes) {
            return None;
        }
        Some((sizes, lanes))
    }

    /// Wrapping is at one size, always written (owner, 2026-09-23: b): a
    /// sum of `i8` and `i16` says which with `to.i16[…]` first.
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
                err(Diagnostic::error(format!("`{}` wraps at one size, and this mixes {}: say which with `to.{}(…)` first", op.spelled(), names.join(" and "), last)).named("wrap_sizes_mixed").at(op_span))
            }
        }
    }

    fn open_size(&self, op: &BinOp, op_span: Span) -> Box<Diagnostic> {
        Box::new(Diagnostic::error(format!("`{}` on an exact answer works at the width of its typed operands, and these have different sizes (or none left to see): say which first — `to.u32(…)`", op.spelled())).named("width_unknown").at(op_span))
    }

    /// A lane-wise operator. The rule is the scalar rule, N times.
    #[allow(clippy::too_many_arguments)]
    fn simd_binary(&mut self, op: &BinOp, op_span: Span, mut l: Expr, mut r: Expr, lt: &Ty, rt: &Ty, span: Span) -> Result<Expr> {
        let (lane, lanes) = match (lt.base(), rt.base()) {
            (Ty::Simd { elem, lanes }, Ty::Simd { elem: e2, lanes: l2 }) => {
                if !self.unify(elem, e2) {
                    return err(Diagnostic::error(format!("`{}` works lane by lane, and these hold different lanes: `{}` and `{}`", op.spelled(), self.spell(elem), self.spell(e2))).named("wrong_type").at(op_span));
                }
                if lanes != l2 && *lanes != 0 && *l2 != 0 {
                    return err(Diagnostic::error(format!("`{}` works lane by lane, and these are different lengths: {} and {}", op.spelled(), lanes, l2)).named("simd_lane_count_mismatch").at(op_span));
                }
                // The side that says its lanes says them for both: a
                // `simd.fill[10]` beside a `simd[i32, 4]` is four int32s.
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
                    return err(Diagnostic::error(format!("`{}` puts a `{}` beside lanes of `{}`: a single value stands for every lane, so it has to be one of them", op.spelled(), self.spell(&scalar), self.spell(elem))).named("wrong_type").at(op_span));
                }
                ((**elem).clone(), *lanes)
            }
            _ => unreachable!(),
        };
        let lane_r = self.resolve(&lane);
        let ok = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => lane_r.is_numeric(),
            BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_) => {
                self.rounding_takes(op, op_span, &lane_r)?;
                lane_r.is_numeric()
            }
            BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::AndBitWrap | BinOp::OrBitWrap | BinOp::XorWrap | BinOp::Shl | BinOp::Shr | BinOp::ShlMasked | BinOp::ShrMasked => lane_r.is_int(),
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => lane_r.is_numeric() || lane_r.base() == &Ty::Bool,
            BinOp::And | BinOp::Or => lane_r.base() == &Ty::Bool,
            BinOp::AndThen | BinOp::OrElse => {
                let plain = if *op == BinOp::AndThen { "and" } else { "or" };
                return err(Diagnostic::error(format!("`{}` works on a single bool: it stops once the answer is known, and a mask has an answer in every lane — `{}` works lane by lane", op.spelled(), plain)).named("operator_wrong_type").at(op_span));
            }
            _ => false,
        };
        if !ok {
            return err(Diagnostic::error(format!("`{}` is not an operator a lane of `{}` has, so a vector of them does not have it either", op.spelled(), self.spell(&lane_r))).named("operator_wrong_type").at(op_span));
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
                    // The nested list is of the element type itself, its
                    // layout the outer list's to hold.
                    if matches!(item.kind, K::List(_)) {
                        item.ty = elem.clone();
                    }
                }
                Ty::Int(IntW::Fixed { .. }) => {
                    if matches!(self.resolve(&item.ty).base(), Ty::Int(IntW::Free(_)) | Ty::Infer(_)) && literal_only(item) && !matches!(item.kind, K::Int(_)) {
                        let to = if plain_group(item) { exact_for(item) } else { elem.base().clone() };
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
    fn as_operand(&mut self, ctx: &mut FnCtx, e: Expr) -> Result<Expr> {
        let mut e = e;
        settle_exact(&mut e)?;
        self.operand_unsettled(ctx, e)
    }

    /// An operand as `as_operand` makes it, an `exactly` literal's width
    /// left to the operator.
    fn operand_unsettled(&mut self, _ctx: &mut FnCtx, e: Expr) -> Result<Expr> {
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
                err(Diagnostic::error(format!("say how `{w}` rounds: `{w}.floor`, `{w}.ceil`, `{w}.trunc` or `{w}.near` — `{meant}` is {why}")).named("rounding_unsaid").at(op_span).fix_edit(op_span, Edit::Replace, meant).fix_edit(op_span, Edit::Replace, format!("{w}.floor")).fix_edit(op_span, Edit::Replace, format!("{w}.ceil")).fix_edit(op_span, Edit::Replace, format!("{w}.trunc")).fix_edit(op_span, Edit::Replace, format!("{w}.near")))
            }
            BinOp::Mod(Round::Near) if fraction => {
                let family = match t.base() {
                    Ty::Deci(_) => "decis",
                    Ty::Brain => "brain16",
                    _ => "bins",
                };
                err(Diagnostic::error(format!("`mod.near` on {family} leaves nothing over: `div.near` there is ordinary division — for what a whole quotient leaves, `mod.floor`, `mod.ceil` or `mod.trunc`")).named("mod_near_on_fraction").at(op_span))
            }
            _ => Ok(()),
        }
    }

    fn numeric_pair(&mut self, ctx: &mut FnCtx, l: Expr, r: Expr, op: &BinOp, op_span: Span) -> Result<(Expr, Expr, Ty)> {
        let lt = self.resolve(&l.ty);
        let rt = self.resolve(&r.ty);
        // A shift still waiting for its width (a bare shifted number,
        // owner 2026-10-07) takes it from here too: from a typed operand
        // beside it, or, beside bare numbers, from where the whole value
        // goes, which they share (`((0 minus 53) shr 'c') mod.floor 23`
        // stored into an `i64`).
        let pending = |this: &Self, t: &Ty| matches!(t, Ty::Infer(_)) && this.widths_pending.iter().any(|(p, _, _)| this.resolve(p) == *t);
        if pending(self, &lt) || pending(self, &rt) {
            let (mut l, mut r) = (l, r);
            let left_waits = pending(self, &lt);
            let (var, other_t) = if left_waits { (lt.clone(), rt.clone()) } else { (rt.clone(), lt.clone()) };
            let other = if left_waits { &mut r } else { &mut l };
            match other_t.base() {
                Ty::Int(IntW::Fixed { .. }) if !other_t.is_exact() => {
                    self.unify(&var, &other_t);
                    return Ok((l, r, other_t.base().clone()));
                }
                Ty::Int(IntW::Free(_)) if literal_only(other) => {
                    retype_literal_arith(other, &var);
                    if let K::Int(_) = other.kind {
                        other.ty = var.clone();
                    }
                    return Ok((l, r, var));
                }
                Ty::Infer(_) if pending(self, &other_t) => {
                    self.unify(&var, &other_t);
                    return Ok((l, r, var));
                }
                _ => {}
            }
            return Err(Box::new(Diagnostic::error(format!("`{}` beside a shift whose width is still open: give the shifted number a type (`to.i64(…)`)", op.spelled())).named("width_unknown").at(op_span)));
        }
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
        // takes the family of its context"). `'n' + 1` with `'n'` a `u64`
        // is a `u64` sum; joining the two made it an `integer`, which
        // cannot hold the upper half of a `u64` and read it as negative.
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
        let plain = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_)) || op.is_comparison();
        // Bare numbers compared with bare numbers are worked out exactly
        // too: `3876992840477302775 + 7782534087963747351 > 0` is true — the
        // sum is one some 64-bit type holds — not a sum held at `i64`
        // that stops (check claude, seed 439).
        if op.is_comparison() {
            if let (Ty::Int(IntW::Free(_)), Ty::Int(IntW::Free(_))) = (lt.base(), rt.base()) {
                if literal_only(&l) && literal_only(&r) {
                    let (mut l, mut r) = (l, r);
                    let (el, er) = (exact_for(&l), exact_for(&r));
                    let both = Ty::exact_at(el.exact_limit().unwrap_or(64).max(er.exact_limit().unwrap_or(64)));
                    retype_literal_arith(&mut l, &both);
                    retype_literal_arith(&mut r, &both);
                    return Ok((l, r, both));
                }
            }
        }
        if plain {
            if let (Ty::Int(a), Ty::Int(b)) = (lt.base(), rt.base()) {
                if matches!(a, IntW::Fixed { .. }) || matches!(b, IntW::Fixed { .. }) {
                    let (mut l, mut r) = (l, r);
                    // In between, the limit is the widest operand's width,
                    // at least 64 bits (owner, 2026-10-03): two `i128`s
                    // add at 128 bits, two `!std_uint256`s at 256.
                    let width = |w: &IntW| match w {
                        IntW::Fixed { bits, .. } if bits & crate::types::EXACT_FLAG != 0 => bits & !crate::types::EXACT_FLAG,
                        IntW::Fixed { bits, .. } => *bits,
                        IntW::Free(_) => 64,
                    };
                    let exact = Ty::exact_at(width(a).max(width(b)));
                    // A group of bare numbers beside one is worked at the
                    // size its own values reach (check claude, seed 32641:
                    // `((-656682024) pow 15) mod.floor 2147483647` passes
                    // 400 bits inside); its value is in between, so it is
                    // held to the limit where it is used (seed 36150).
                    for (side, w) in [(&mut l, a), (&mut r, b)] {
                        if literal_only(side) && matches!(w, IntW::Free(_)) {
                            let own = exact_for(side);
                            let at = if own.exact_limit() > exact.exact_limit() { own } else { exact.clone() };
                            retype_literal_arith(side, &at);
                        }
                    }
                    return Ok((l, r, exact));
                }
            }
        }
        // Bare numbers alone are worked out exactly (dc83c05), a group of
        // them as `div`'s or `mod`'s other side too: `'p' mod (((221 x 2)
        // mod 5) + 1)` with 'p' a `u8` divides by 3, where the group took
        // `u8` and its `221 x 2` was refused (check claude, seed 4698).
        // The division is then exact as well, and lands where it is stored
        // — as `'p' + (…)` already does. A lone literal still takes the
        // other side's type.
        if matches!(op, BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_)) {
            let group = |e: &Expr, t: &Ty| matches!(t.base(), Ty::Int(IntW::Free(_))) && literal_only(e) && !is_int_literal(e);
            let fixed = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Fixed { .. }));
            if (group(&l, &lt) && fixed(&rt)) || (group(&r, &rt) && fixed(&lt)) {
                let (mut l, mut r) = (l, r);
                let exact = if group(&l, &lt) { exact_for(&l) } else { exact_for(&r) };
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
                return err(Diagnostic::error(format!("a negative number beside a `{}`: an unsigned number is never negative, and a bare number takes the family of the other side", fam)).named("negative_unsigned").at(lit.span));
            }
        }
        if !lt.is_numeric() {
            return err(Diagnostic::error(format!("`{}` works on numbers, and its left side is {}", op.spelled(), crate::types::a_type(&self.spell(&lt)))).named("operator_wrong_type").at(l.span));
        }
        if !rt.is_numeric() {
            return err(Diagnostic::error(format!("`{}` works on numbers, and its right side is {}", op.spelled(), crate::types::a_type(&self.spell(&rt)))).named("operator_wrong_type").at(r.span));
        }
        let ty = match (lt.base(), rt.base()) {
            _ if adopt.is_some() => adopt.clone().unwrap(),
            (Ty::Int(a), Ty::Int(b)) => Ty::Int(join_int(*a, *b)),
            (Ty::Bin(a), Ty::Bin(b)) => Ty::Bin(join_bin(*a, *b)),
            (Ty::Deci(a), Ty::Deci(b)) => Ty::Deci((*a).max(*b)),
            _ => return err(self.mixed_families(Diagnostic::error(format!("`{}` between a `{}` and a `{}`: there is no implicit conversion between numeric families — convert one side with `to.{}(…)`", op.spelled(), self.spell(&lt), self.spell(&rt), self.spell(lt.base()))).named("no_implicit_conversion").at(op_span), (&lt, l.span), (&rt, r.span))),
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
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_) | BinOp::Pow => {}
            _ if op.is_comparison() => {}
            _ => return err(Diagnostic::error(format!("`{}` works on whole numbers, and a brain16 is not one", op.spelled())).named("needs_integer").at(op_span)),
        }
        let (lt, rt) = (self.resolve(&l.ty), self.resolve(&r.ty));
        if lt.is_brain() && rt.is_brain() {
            return Ok((l, r, Ty::Brain));
        }
        let other = if lt.is_brain() { &rt } else { &lt };
        err(Diagnostic::error(format!("`{}` between a brain16 and {}: the brain float is a family of its own — convert one side first, `to.brain16(…)` or `to.f32(…)`", op.spelled(), article(&self.spell(other)))).named("ai::brain_mixed").at(op_span))
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
                return err(Diagnostic::error(format!("a deci power's exponent is an integer or a whole literal, and this one is {}: roots are std's math", crate::types::a_type(&self.spell(&rt)))).named("money::power_not_whole").at(r.span));
            }
            return err(Diagnostic::error("a deci power's exponent is an integer or a whole literal: the deci goes on the left of `xx`").named("money::power_not_whole").at(op_span));
        }
        match (lt.base(), rt.base()) {
            (Ty::Deci(b), other) if *b != EXACT_FREE && !matches!(other, Ty::Deci(x) if *x != EXACT_FREE) && deci_group(&r) => into_deci(&mut r, *b)?,
            (other, Ty::Deci(b)) if *b != EXACT_FREE && !matches!(other, Ty::Deci(x) if *x != EXACT_FREE) && deci_group(&l) => into_deci(&mut l, *b)?,
            // Two exact literals with nothing else to say: both `d64`.
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
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_) => {}
            _ if op.is_comparison() => {}
            _ => return err(Diagnostic::error(format!("`{}` works on whole numbers, and a deci is not one", op.spelled())).named("needs_integer").at(op_span)),
        }
        match (lt.base(), rt.base()) {
            (Ty::Deci(a), Ty::Deci(b)) if a == b => Ok((l, r, Ty::Deci(*a))),
            (Ty::Deci(a), Ty::Deci(b)) => {
                let wide = format!("d{}", (*a).max(*b));
                err(Diagnostic::error(format!("`{}` between a d{} and a d{}: say which width, `to.{}(…)`", op.spelled(), a, b, wide)).named("money::deci_mixed").at(op_span))
            }
            (Ty::Deci(b), _) | (_, Ty::Deci(b)) => err(Diagnostic::error(format!("`{}` between a deci and another number family: there is no implicit conversion between numeric families; `to.d{}(…)`", op.spelled(), b)).named("money::deci_mixed").at(op_span)),
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
        err(Diagnostic::error(format!("`{}` compares two values of one type: the left is a `{}`, the right a `{}`", op.spelled(), self.spell(lt), self.spell(&rt))).named("wrong_type").at(op_span))
    }

    fn user_op(&mut self, ctx: &mut FnCtx, spelling: &str, op_span: Span, lhs: &ast::Expr, rhs: &ast::Expr, span: Span) -> Result<Expr> {
        let l = self.expr(ctx, lhs)?;
        let lt = self.resolve(&l.ty);
        // A path picks only that Kit's operator (owner, 2026-10-03).
        let (kit, name) = match spelling.split_once("::") {
            Some((kit, name)) => {
                let here = self.items.kit_root(ctx.module);
                let Some(root) = self.items.kit_deps.get(&here).and_then(|d| d.get(kit)).copied() else {
                    return err(Diagnostic::error(format!("no kit with the prefix `{}`: a kit this one uses is listed under `my kits:` in its `CyJet.toml`, and its own `CyJet.toml` says `prefix = {}`", kit, kit)).named("kit_not_listed").at(op_span));
                };
                (Some((kit, root)), name)
            }
            None => (None, spelling),
        };
        let Some(fn_id) = self.find_op_in(name, lt.base(), kit.map(|l| l.1)) else {
            if let Some((kit, _)) = kit {
                return err(Diagnostic::error(format!("the kit `{}` has no operator `{}` for {}: a path picks only that kit's operator", kit, name, crate::types::a_type(&self.spell(&lt)))).named("unknown_operator").at(op_span));
            }
            // A glyph free to declare has its hint in spellings.cyr.
            let hint = match crate::spellings::free_op_hint(spelling) {
                Some(h) => h,
                None => match spelling {
                    "&+" | "&-" | "+%" | "-%" | "*%" => "wrapping arithmetic is `plus.wrap`, `minus.wrap`, `times.wrap`, `pow.wrap`",
                    _ => "an operator between two values is one the language owns (`plus`, `times`, `mod.floor`, …) or one declared with `op`",
                },
            };
            let mut d = Diagnostic::error(format!("`{}` is not an operator here; {}", spelling, hint)).named(if crate::spellings::free_op_hint(spelling).is_some() { "old_spelling" } else { "unknown_operator" }).at(op_span);
            // The operators the hint names in backticks; `not` and `neg`
            // take one value, so they are not this operator's place.
            for w in hint.split('`').skip(1).step_by(2).filter(|w| !w.contains(|c: char| c == ' ' || c == '$' || c == '(') && !matches!(*w, "not" | "neg" | "op")) {
                d = d.fix_edit(op_span, Edit::Replace, w);
            }
            return err(d);
        };
        // Bare, it may be more than one Kit's (owner, 2026-10-06): refused
        // where it is written, and this Kit's own does not win.
        if kit.is_none() {
            let here = self.items.kit_root(ctx.module);
            let mut roots: Vec<crate::resolve::ModuleId> = Vec::new();
            for id in self.ops_matching(name, lt.base(), None, false) {
                let r = self.items.kit_root(self.items.fns[id.0 as usize].module);
                if !roots.contains(&r) {
                    roots.push(r);
                }
            }
            if roots.len() > 1 {
                let mut named: Vec<String> = roots.iter().map(|r| match self.items.kit_deps.get(&here).and_then(|d| d.iter().find(|(_, x)| *x == r)) {
                    Some((p, _)) => format!("`{}::{}`", p, name),
                    None => format!("`my.{}` (this kit's)", name),
                }).collect();
                named.sort();
                return err(Diagnostic::error(format!("`{}` could be any of {}: write the path", name, named.join(", "))).named("name_ambiguous").at(op_span));
            }
        }
        // Another Kit's word operator follows its mark (owner,
        // 2026-10-06): `bare.kit_prefix.never` needs the path, `bare.kit_prefix.only` refuses it.
        let decl = &self.items.fns[fn_id.0 as usize];
        let (here, there) = (self.items.kit_root(ctx.module), self.items.kit_root(decl.module));
        if here != there {
            match (kit, self.items.bare_marks.get(&decl.name_span)) {
                (None, Some(ast::Bare::Never)) => {
                    let prefix = self.items.kit_deps.get(&here).and_then(|d| d.iter().find(|(_, r)| **r == there)).map(|(p, _)| p.clone()).unwrap_or_default();
                    return err(Diagnostic::error(format!("`{n}` is `{p}`'s and `bare.kit_prefix.never`: write `{p}::{n}`", n = name, p = prefix)).named("kit_name_needs_path").at(op_span));
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
                        return err(Diagnostic::error(format!("`{}` lends its operand: `give` is not wanted here", spelling)).named("verb_wrong_for_parameter").at(side.span).fix_edit(side.span, Edit::Verb, ""));
                    }
                    let place = self.place(ctx, expr, PlaceUse::Give)?;
                    Arg::Give(place)
                }
                AK::Verb { verb, .. } => return err(Diagnostic::error(format!("an operand takes no `{}`: operands are lent unless the operator says `give`", verb.spelled())).named("verb_wrong_for_parameter").at(side.span).fix_edit(side.span, Edit::Verb, "")),
                _ => {
                    let v = self.expr(ctx, side)?;
                    let vt = self.resolve(&v.ty);
                    if gives {
                        match &v.kind {
                            K::Read(_) => {
                                let n = pdecl.name.clone();
                                return err(Diagnostic::error(format!("`{}` takes '{}' by value: write `give {}` so the value moves in", spelling, n, self.expr_name(ctx, &v))).named("owned_value_bare").at(side.span).fix_edit(v.span, Edit::Verb, "give"));
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
                return err(Diagnostic::error(format!("an operand takes no `{}`: a generic operator's operands are lent", verb.spelled())).named("verb_wrong_for_parameter").at(side.span).fix_edit(side.span, Edit::Verb, ""));
            }
            vals.push(self.expr(ctx, side)?);
        }
        let free = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_)) | Ty::Infer(_));
        for (v, pt) in vals.iter().zip(&ptys) {
            let vt = self.resolve(&v.ty);
            if !free(&vt) && !self.unify(&vt, pt) {
                return err(Diagnostic::error(format!("one blank, one type: `{}` takes both operands as the same `<T>`, and this is a `{}` beside a `{}`", spelling, self.spell(&vt), self.spell(&self.resolve(pt)))).named("generic_filled_inconsistently").at(v.span));
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
                return err(Diagnostic::error(format!("the blank `<{}>` of `{}` is not filled: neither operand says what it is", decl.blanks[i].0, spelling)).named("generic_unfilled").at(span));
            }
            for iface in decl.blanks[i].1.clone() {
                if let Err(reason) = self.satisfies(&r, iface) {
                    let iname = self.items.ifaces[iface.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` asks for `<{}>: {}`", self.spell(&r), iname, reason, spelling, decl.blanks[i].0, iname)).named("bound_not_satisfied").at(span));
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
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` needs `<{}>: {}`", self.spell(&r), iname, reason, owner, names.get(i).cloned().unwrap_or_default(), iname)).named("bound_not_satisfied").at(span));
                }
            }
        }
        Ok(())
    }

    /// How many places a value can have (owner, 2026-10-17): a literal's as
    /// written (a percent's in `%`), round.X's `places:`, a value whose
    /// type keeps some, the larger of `+` and `-`, the sum of `x`.
    fn places_in(&self, e: &Expr) -> PlacesOf {
        let in_percent = percent_inner(&self.resolve(&e.ty)).is_some();
        let literal = |n: u32| PlacesOf::Literal(if in_percent { n.saturating_sub(2) } else { n });
        match &e.kind {
            K::Deci { bits, value } => {
                let (_, _, exp) = crate::deci::decode(*value, *bits);
                return literal(if exp < 0 { (-exp) as u32 } else { 0 });
            }
            K::Bin(t) => return literal(t.split_once('.').map_or(0, |(_, f)| f.replace('_', "").len() as u32)),
            K::Int(_) => return PlacesOf::Literal(0),
            K::Unary { op: UnOp::Neg, expr } => return self.places_in(expr),
            K::Binary { op, lhs, rhs, .. } if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul) => {
                let n = |p: PlacesOf| match p {
                    PlacesOf::Literal(n) | PlacesOf::Can(n) => Some(n),
                    PlacesOf::Unknown => None,
                };
                return match (n(self.places_in(lhs)), n(self.places_in(rhs))) {
                    (Some(a), Some(b)) if *op == BinOp::Mul => PlacesOf::Can(a + b),
                    (Some(a), Some(b)) => PlacesOf::Can(a.max(b)),
                    _ => PlacesOf::Unknown,
                };
            }
            // std's round.X: `places: N`, or a whole number without it.
            K::Call { callee: hir::Callee::Fn(inst), args, .. } => {
                let decl = &self.items.fns[self.instances[inst.0 as usize].key.fn_id.0 as usize];
                let std = self.items.nerve_cord.is_some_and(|c| self.items.kit_root(decl.module) == c);
                if std && matches!(decl.name.as_str(), "near" | "half_up" | "floor" | "ceil" | "trunc") && decl.path.contains("round") {
                    return match args.get(1) {
                        None => PlacesOf::Can(0),
                        Some(hir::Arg::Value(Expr { kind: K::Int(n), .. })) => PlacesOf::Can((*n).clamp(0, u32::MAX as i128) as u32),
                        Some(_) => PlacesOf::Unknown,
                    };
                }
            }
            _ => {}
        }
        match places_max(&self.resolve(&e.ty)) {
            Some(n) => PlacesOf::Can(n),
            None => PlacesOf::Unknown,
        }
    }

    /// `calm` (or `calm[…]`) where a `Mood` is asked for, when `Mood`'s
    /// values may drop its name (`bare.enum_prefix.optional` or `.only`);
    /// `None` when it is not one.
    fn bare_case(&mut self, ctx: &mut FnCtx, e: &ast::Expr, want: &Ty) -> Option<Result<Expr>> {
        let Ty::Enum { id, .. } = self.resolve(want).base().clone() else { return None };
        let (name, payload) = match &e.kind {
            AK::Path(p) if p.kit.is_none() && p.segs.len() == 1 => (&p.segs[0], None),
            AK::Call { path, args, otherwise: None } if path.kit.is_none() && path.segs.len() == 1 => (&path.segs[0], Some(&args[..])),
            _ => return None,
        };
        let info = &self.items.enums[id.0 as usize];
        if !info.cases.iter().any(|c| c.name == name.name) {
            return None;
        }
        if !matches!(self.items.enum_bare_marks.get(&info.span), Some(ast::Bare::Optional | ast::Bare::Only)) {
            return None;
        }
        Some(self.enum_case(ctx, id, name, payload, e.span))
    }

    /// `Mood.calm` written with its name: always allowed since the marks
    /// that could forbid it went (owner, 2026-10-22).
    fn enum_named_ok(&self, _id: EnumId, _case: &ast::Ident, _span: Span) -> Result<()> {
        Ok(())
    }

    fn enum_case(&mut self, ctx: &mut FnCtx, id: EnumId, case: &ast::Ident, payload: Option<&[AArg]>, span: Span) -> Result<Expr> {
        let info = self.items.enums[id.0 as usize].clone();
        let Some(ci) = info.cases.iter().position(|c| self.items.same_name(&self.items.case_cases, info.span, &c.name, &case.name)) else {
            return err(Diagnostic::error(format!("`{}` has no case `{}`: its cases are {}", info.name, case.name, info.cases.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", "))).named("unknown_case").at(case.span).fix_close(case.span, &case.name, info.cases.iter().map(|c| c.name.as_str())));
        };
        let args: Vec<Ty> = info.blanks.iter().map(|_| self.fresh()).collect();
        let expected = info.cases[ci].payload.as_ref().map(|t| t.subst(&args));
        let payload = match (payload, expected) {
            (None, None) => None,
            (Some(a), Some(t)) => {
                if a.len() != 1 {
                    return err(Diagnostic::error(format!("`{}.{}` carries one value", info.name, case.name)).named("case_data_form").at(span));
                }
                let AArg::Positional(e) = &a[0] else {
                    return err(Diagnostic::error(format!("`{}.{}` carries one value, not a field", info.name, case.name)).named("case_data_form").at(span));
                };
                let v = self.expr(ctx, e)?;
                let v = self.expect_store(ctx, v, &t, &format!("`{}`'s data", case.name))?;
                self.check_stored_value(ctx, &v, &t)?;
                Some(Box::new(self.value_arg(v)))
            }
            (None, Some(t)) => return err(Diagnostic::error(format!("`{}.{}` carries {}: write `{}.{}(…)` with the value", info.name, case.name, crate::types::a_type(&self.spell(&t)), info.name, case.name)).named("data_missing").at(span)),
            (Some(_), None) => return err(Diagnostic::error(format!("`{}.{}` carries nothing; write it bare — `{}.{}`", info.name, case.name, info.name, case.name)).named("carries_nothing").at(span)),
        };
        self.kept_bounds(&info.name, &info.blanks, &info.bounds, &args, span)?;
        Ok(Expr { kind: K::EnumCase { id, case: ci, payload }, ty: Ty::Enum { id, args }, span })
    }

    fn construct(&mut self, ctx: &mut FnCtx, id: StructId, args: &[AArg], span: Span) -> Result<Expr> {
        let info = self.items.structs[id.0 as usize].clone();
        if info.opaque {
            return err(Diagnostic::error(format!("`{n}` is C's and opaque (owner, 2026-10-10): it is only pointed at, `ptr.mut[{n}]`, never built", n = info.name)).named("cnc::opaque_struct_held").at(span));
        }
        let targs: Vec<Ty> = info.blanks.iter().map(|_| self.fresh()).collect();
        let mut given: Vec<Option<Arg>> = vec![None; info.fields.len()];
        for a in args {
            match a {
                AArg::Named(name, e) => {
                    let Some(fi) = info.fields.iter().position(|f| self.items.same_name(&self.items.case_fields, info.span, &f.name, &name.name)) else {
                        return err(Diagnostic::error(format!("`{}` has no field `{}`: its fields are {}", info.name, name.name, info.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", "))).named("unknown_field").at(name.span).fix_close(name.span, &name.name, info.fields.iter().map(|f| f.name.as_str())));
                    };
                    if given[fi].is_some() {
                        return err(Diagnostic::error(format!("field `{}` is given twice", name.name)).named("given_twice").at(name.span));
                    }
                    let t = info.fields[fi].ty.subst(&targs);
                    let v = self.expr(ctx, e)?;
                    let v = self.expect_store(ctx, v, &t, &format!("field `{}`", name.name))?;
                    self.check_stored_value(ctx, &v, &t)?;
                    given[fi] = Some(self.value_arg(v));
                }
                AArg::Positional(e) => return err(Diagnostic::error(format!("a struct is built by field name: `{}[{} = …]`", info.name, info.fields.first().map(|f| f.name.as_str()).unwrap_or("field"))).named("field_unnamed").at(e.span)),
                AArg::Pair(k, _) => return err(Diagnostic::error(format!("a struct is built by field name: `{}[{} = …]`", info.name, info.fields.first().map(|f| f.name.as_str()).unwrap_or("field"))).named("field_unnamed").at(k.span)),
            }
        }
        // A union is built from one field (owner, 2026-10-19); the others
        // are left as nothing, and the code generator writes the one given.
        if info.union {
            if given.iter().filter(|g| g.is_some()).count() != 1 {
                return err(Diagnostic::error(format!("`{}` is a `layout.C union`: it holds one field at a time, so give exactly one: `{}[{} = …]`", info.name, info.name, info.fields.first().map(|f| f.name.as_str()).unwrap_or("field"))).named("union_one_field").at(span));
            }
            let fields = given.into_iter().map(|g| g.unwrap_or(Arg::Value(Expr { kind: K::Nothing, ty: Ty::Nothing, span }))).collect();
            return Ok(Expr { kind: K::Construct { id, fields }, ty: Ty::Struct { id, args: targs }, span });
        }
        let missing: Vec<&str> = info.fields.iter().enumerate().filter(|(i, _)| given[*i].is_none()).map(|(_, f)| f.name.as_str()).collect();
        if !missing.is_empty() {
            return err(Diagnostic::error(format!("`{}` is missing {}: every field must be given — there are no zero values", info.name, missing.iter().map(|m| format!("`{}`", m)).collect::<Vec<_>>().join(", "))).named("field_missing").at(span));
        }
        let fields = given.into_iter().map(|g| g.unwrap()).collect();
        self.kept_bounds(&info.name, &info.blanks, &info.bounds, &targs, span)?;
        Ok(Expr { kind: K::Construct { id, fields }, ty: Ty::Struct { id, args: targs }, span })
    }

    fn map_literal(&mut self, ctx: &mut FnCtx, args: &[AArg], span: Span) -> Result<Expr> {
        if args.is_empty() {
            return err(Diagnostic::error("an empty map is `map.empty`; `map[]` builds nothing").named("empty_map_literal").at(span).fix_edit(span, Edit::Replace, "map.empty"));
        }
        let kt = self.fresh();
        let vt = self.fresh();
        let mut pairs = Vec::new();
        let mut seen_keys: Vec<Expr> = Vec::new();
        for a in args {
            let AArg::Pair(k, v) = a else {
                return err(Diagnostic::error("a map literal pairs each key with a value: `map[\"ann\" = 5551, \"bob\" = 5552]`").named("map_entry_form").at(a.span()));
            };
            let kv = self.expr(ctx, k)?;
            let kv = self.expect_store(ctx, kv, &kt, "the key type")?;
            self.items.check_key_type(&self.resolve(&kv.ty), k.span)?;
            self.check_stored_value(ctx, &kv, &kt)?;
            for prev in &seen_keys {
                if same_literal(prev, &kv) {
                    return err(Diagnostic::error("this key is written twice in the literal").named("duplicate_map_key").at(k.span));
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
        // `std::round`, `floor`, `ceil`, `trunc` became one family (owner,
        // 2026-10-17): each refused from spellings.cyr's row.
        if path.kit.as_ref().is_some_and(|l| l.name == "std") {
            if let [one] = path.segs.as_slice() {
                if matches!(one.name.as_str(), "round" | "floor" | "ceil" | "trunc") {
                    if let Some(o) = crate::spellings::old_word(&format!("std::{}", one.name), Some("special")) {
                        // `round.near($x)`: the path is the part before its `(`.
                        let mut d = Diagnostic::error(o.dated()).named("old_spelling").at(path.span);
                        for w in &o.write {
                            d = d.fix_edit(path.span, Edit::Replace, w.split('(').next().unwrap_or(w).to_string());
                        }
                        return err(d);
                    }
                }
            }
            if let [r, way] = path.segs.as_slice() {
                if r.name == "round" && !matches!(way.name.as_str(), "near" | "half_up" | "floor" | "ceil" | "trunc") {
                    let mut d = Diagnostic::error(format!("`std::round.{}` is not a way to round: `round.near`, `round.half_up`, `round.floor`, `round.ceil` or `round.trunc`", way.name)).named("unknown_rounding").at(path.span);
                    for w in ["near", "half_up", "floor", "ceil", "trunc"] {
                        d = d.fix_edit(way.span, Edit::Replace, w);
                    }
                    return err(d);
                }
            }
        }
        // Another kit's names always take its path, std's too (owner,
        // 2026-10-22); inside std they are its own. The built-ins std owns
        // are reached with `std::` taken off.
        let via_std = std::mem::take(&mut self.via_std);
        if path.kit.as_ref().is_some_and(|l| l.name == "std") && std_builtin(path) {
            let plain = ast::Path { kit: None, segs: path.segs.clone(), span: path.span };
            self.via_std = true;
            return self.call(ctx, &plain, args, otherwise, span, used);
        }
        if path.kit.is_none() && (std_name(path) || std_builtin(path)) && self.items.lookup_plain(path, ctx.module).is_err() {
            if !(via_std || self.in_std(ctx)) {
                return err(Diagnostic::error(std_bare_message(path)).named("std_name_needs_path").at(path.span).fix_edit(path.span, Edit::Insert, "std::"));
            }
            if std_name(path) {
                let with = ast::Path { kit: Some(ast::Ident { name: "std".into(), span: path.span }), segs: path.segs.clone(), span: path.span };
                return self.call(ctx, &with, args, otherwise, span, used);
            }
        }
        // The Nerve cord, when the program lists it, takes over `cord::`
        // name by name (owner, 2026-10-14); what it has not written yet is
        // the built-ins' still.
        if path.kit.as_ref().is_some_and(|l| l.name == "std") {
            let inner = ast::Path { kit: None, segs: path.segs.clone(), span: path.span };
            match self.items.nerve_cord_item(&inner).map(|(i, _)| i) {
                Some(ItemRef::Fn(id)) => return self.fn_call(ctx, id, args, otherwise, span, used, path),
                Some(ItemRef::ExternFn(id)) => return self.extern_call(ctx, id, args, otherwise, span, used),
                Some(ItemRef::Struct(id)) => {
                    if otherwise.is_some() {
                        return err(Diagnostic::error("building a struct value cannot fail; drop the `otherwise`").named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                    }
                    return self.construct(ctx, id, args, span);
                }
                _ => {}
            }
        }
        // The prints are one family (owner, dc7e2e4): the old names say the
        // new ones, with std or without it.
        if path.kit.as_ref().map(|l| l.name == "std").unwrap_or(false) {
            let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
            let renamed = match segs.first() {
                Some(&"printu") => Some("print.useful"),
                Some(&"printf") => Some("print.format"),
                _ => None,
            };
            if let Some(new) = renamed {
                let rest = segs[1..].join(".");
                let (old, new) = if rest.is_empty() { (segs[0].to_string(), new.to_string()) } else { (format!("{}.{}", segs[0], rest), format!("{}.{}", new, rest)) };
                return err(Diagnostic::error(format!("std::{} is spelled std::{} now", old, new)).named("old_spelling").at(span).fix_edit(path.span, Edit::Replace, format!("std::{}", new)));
            }
        }
        // Every print says its form (owner, 2026-10-02): `.stored` or
        // `.useful`, after `.weird` when there is one.
        if path.kit.as_ref().map(|l| l.name == "std").unwrap_or(false) && path.segs.first().is_some_and(|s| s.name == "print") {
            let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
            match segs.as_slice() {
                ["print"] => {
                    return err(Diagnostic::error("say which form a print shows: `std::print.stored(…)` (what is stored) or `std::print.useful(…)` (the useful value) (owner, 2026-10-02)").named("print_form_unsaid").at(span).fix_edit(path.span, Edit::Swap(vec!["std::print".into()]), "std::print.useful").fix_edit(path.span, Edit::Swap(vec!["std::print".into()]), "std::print.stored"));
                }
                ["print", stream @ ("stdout" | "stderr")] => {
                    return err(Diagnostic::error(format!("say which form a print shows: `std::print.stored.{}(…)` (what is stored) or `std::print.useful.{}(…)` (the useful value) (owner, 2026-10-02)", stream, stream)).named("print_form_unsaid").at(span).fix_edit(path.span, Edit::Swap(vec![format!("print.{}", stream)]), format!("print.useful.{}", stream)).fix_edit(path.span, Edit::Swap(vec![format!("print.{}", stream)]), format!("print.stored.{}", stream)));
                }
                ["print", "weird", stream @ ("stdout" | "stderr")] => {
                    return err(Diagnostic::error(format!("say which form a print shows: `std::print.weird.stored.{} <- …` (what is stored) or `std::print.weird.useful.{} <- …` (the useful value) (owner, 2026-10-02)", stream, stream)).named("print_form_unsaid").at(span));
                }
                ["print", f @ ("stored" | "useful"), "weird", rest @ ..] => {
                    return err(Diagnostic::error(format!("`.weird` comes first, the strongest modifier first: `std::print.weird.{}.{} <- …`", f, rest.join("."))).named("print_weird_form").at(span).fix_edit(path.span, Edit::Replace, if rest.is_empty() { format!("std::print.weird.{}", f) } else { format!("std::print.weird.{}.{}", f, rest.join(".")) }));
                }
                _ => {}
            }
        }
        // Moved into the language (owner, 2026-10-02).
        if path.kit.as_ref().map(|l| l.name == "std").unwrap_or(false) {
            let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
            if let [w @ ("div" | "mod"), "trunc"] = segs.as_slice() {
                let d = Diagnostic::error(format!("`std::{w}.trunc` is an operator in the language now: `$a {w}.trunc $b`")).named("old_spelling").at(span);
                // `std::div.trunc(a, b)`: `(a div.trunc b)`.
                if let [a, b] = args {
                    let (a, b) = (a.span(), b.span());
                    let cut = |from: u32, to: u32| Span::new(span.file, from as usize, to as usize);
                    return err(d.fix_edit(cut(span.start, a.start), Edit::Replace, "(").fix_also(cut(a.end, b.start), Edit::Replace, format!(" {w}.trunc ")).fix_also(cut(b.end, span.end), Edit::Replace, ")"));
                }
                return err(d);
            }
        }
        // A channel is the MCPS limb's, written bare, never std's (design
        // claude, 2026-10-24).
        if path.kit.as_ref().is_some_and(|l| l.name == "std") && path.segs.first().is_some_and(|s| s.name == "channel") {
            let rest = path.segs.iter().skip(1).map(|s| s.name.as_str()).collect::<Vec<_>>().join(".");
            return err(Diagnostic::error(format!("`channel` is the MCPS limb's, not std's: write `channel.{}(…)`", if rest.is_empty() { "send" } else { &rest })).named("unknown_std_name").at(path.span));
        }
        // A writer's way is `.whole` or `.split` (owner, 2026-10-24).
        if path.kit.as_ref().is_some_and(|l| l.name == "std") {
            let name = path.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(".");
            if name.starts_with("print.writer.") && !BUILT_CORD.contains(&name.as_str()) {
                return err(Diagnostic::error("a writer is `std::print.writer`, `.whole` or `.split`, then `.stderr` for stderr (owner, 2026-10-24)").named("writer_way_unknown").at(path.span).fix_close(path.span, &path.spelled(), ["std::print.writer", "std::print.writer.whole", "std::print.writer.split", "std::print.writer.stderr"]));
            }
        }
        if self.std_stubs && path.kit.as_ref().map(|l| l.name == "std").unwrap_or(false) {
            return self.std_stub(ctx, path, args, otherwise, span, used);
        }
        // Of cord, `print` and `exit` are built (owner, 2026-10-09: "`b`,
        // not the entire fucking neuron"); each other print is refused by
        // name, and the rest of cord as before.
        if path.kit.as_ref().is_some_and(|l| l.name == "std") {
            let name = path.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(".");
            if BUILT_CORD.contains(&name.as_str()) {
                // With no OS, the target's output or exit is what they
                // call: made into code, as a call would make it.
                if crate::target::bare() {
                    let board = if name == "exit" { self.items.board_exit } else { self.items.board_output.map(|(id, _)| id) };
                    if let Some(id) = board {
                        self.instance(crate::check::InstKey { fn_id: id, blanks: Vec::new(), bundle: None, owning: Vec::new() });
                    }
                }
                return self.std_stub(ctx, path, args, otherwise, span, used);
            }
            if name == "format" || name.starts_with("print.") {
                let shown = name.replace("auto_newline", "auto_newln");
                return err(Diagnostic::error(format!("`std::{}` is not built yet: of std, kayo builds `std::print.useful`, `std::print.stored` (each with `.stdout` or `.stderr`), `std::print.useful.auto_newln`, the two `.weird` forms and `std::exit` (owner, 2026-10-09)", shown)).named(if shown.ends_with("format") { "format_text_wrong" } else { "not_built_yet" }).at(path.span));
            }
        }
        // Language operations first.
        if path.kit.is_none() {
            if let Some(b) = self.builtin_head(path, span)? {
                return self.builtin_call(ctx, b, path, args, otherwise, span, used);
            }
        }
        // A type's own function: `Counter.next[…]` (owner, 2026-10-09).
        if let Some(id) = self.items.lookup_type_fn(path, ctx.module)? {
            return self.fn_call(ctx, id, args, otherwise, span, used, path);
        }
        // An enum case with data: `Status.claimed["x"]`.
        if path.segs.len() >= 2 && path.kit.is_none() {
            let prefix = ast::Path { kit: None, segs: path.segs[..path.segs.len() - 1].to_vec(), span: path.span };
            if let Ok(ItemRef::Enum(id)) = self.items.lookup_path(&prefix, ctx.module) {
                if otherwise.is_some() {
                    return err(Diagnostic::error("building an enum value cannot fail; drop the `otherwise`").named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                self.enum_named_ok(id, path.last(), span)?;
                return self.enum_case(ctx, id, path.last(), Some(args), span);
            }
        }
        let item = match self.items.lookup_path(path, ctx.module) {
            Ok(i) => i,
            Err(e) => {
                if path.segs.len() == 1 && path.kit.is_none() && self.items.ops.contains_key(path.head()) {
                    return err(Diagnostic::error(format!("`{}` is an operator; it goes between two values: `$a {} $b`, never `{}(…)`", path.head(), path.head(), path.head())).named("operator_called_as_function").at(span));
                }
                if path.segs.len() == 1 && path.kit.is_none() && args.iter().any(|a| matches!(a, AArg::Named(..))) && e.sentence.starts_with("there is no") {
                    return err(Diagnostic::error(format!("there is no struct named `{}` here", path.head())).named("unknown_struct").at(path.span).fix_close(path.span, path.head(), self.items.structs.iter().map(|s| s.name.as_str())));
                }
                return Err(e);
            }
        };
        match item {
            ItemRef::Struct(id) => {
                if otherwise.is_some() {
                    return err(Diagnostic::error("building a struct value cannot fail; drop the `otherwise`").named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                self.construct(ctx, id, args, span)
            }
            // Nothing calls a `test fn` (owner, 2026-09-29).
            ItemRef::Fn(id) if self.items.fns[id.0 as usize].test => {
                err(Diagnostic::error(format!("`{}` is a `test fn`: `kayo test` runs it, and nothing else calls it", path.spelled())).named("test_fn_misused").at(span))
            }
            ItemRef::Fn(id) => self.fn_call(ctx, id, args, otherwise, span, used, path),
            ItemRef::ExternFn(id) => self.extern_call(ctx, id, args, otherwise, span, used),
            ItemRef::Enum(_) => err(Diagnostic::error(format!("`{}` is an enum: a value is one of its cases — `{}.case` or `{}.case[data]`", path.spelled(), path.spelled(), path.spelled())).named("not_a_function").at(span)),
            other => err(Diagnostic::error(format!("`{}` is a {}, not a function", path.spelled(), other.what())).named(if path.kit.as_ref().is_some_and(|l| l.name == "std") && matches!(path.segs.last().map(|s| s.name.as_str()), Some("upper" | "lower" | "title" | "fold")) { "case_mapping_unsaid" } else { "not_a_function" }).at(span)),
        }
    }

    pub(crate) fn fn_call(&mut self, ctx: &mut FnCtx, id: FnId, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool, path: &ast::Path) -> Result<Expr> {
        let set = self.items.versions.get(&id).cloned().unwrap_or_else(|| vec![id]);
        // Labels and defaults (owner, 2026-10-21): each version's arguments
        // in its parameters' order, a left-out one its default.
        let first = &self.items.fns[set[0].0 as usize];
        let labelled = first.labels.is_some() || !first.defaults.is_empty() || args.iter().any(|a| matches!(a, AArg::Positional(ast::Expr { kind: AK::Label(..), .. })));
        if labelled {
            let (plain, given) = self.call_labels(set[0], args, path)?;
            let mut shaped: Vec<(FnId, Vec<AArg>)> = Vec::new();
            let mut refused = None;
            for f in &set {
                match self.shape_args(*f, &plain, &given, span, path) {
                    Ok(a) => shaped.push((*f, a)),
                    Err(e) => {
                        refused.get_or_insert(e);
                    }
                }
            }
            if shaped.is_empty() {
                return Err(refused.expect("a version was refused"));
            }
            let id = if set.len() > 1 {
                let cands: Vec<(FnId, &[AArg])> = shaped.iter().map(|(f, a)| (*f, a.as_slice())).collect();
                self.pick_version(ctx, &cands, span, path)?
            } else {
                shaped[0].0
            };
            let args = shaped.into_iter().find(|(f, _)| *f == id).map(|(_, a)| a).unwrap_or_default();
            return self.fn_call_one(ctx, id, &args, otherwise, span, used, path);
        }
        // A function with versions (owner, 2026-10-20): the one this call's
        // argument count and types pick.
        let id = if set.len() > 1 {
            let cands: Vec<(FnId, &[AArg])> = set.iter().map(|f| (*f, args)).collect();
            self.pick_version(ctx, &cands, span, path)?
        } else {
            id
        };
        self.fn_call_one(ctx, id, args, otherwise, span, used, path)
    }

    /// A call's arguments without a label, in place, and its labelled ones
    /// (owner, 2026-10-21): the arguments without one come first; each label
    /// is one of the function's; `labels.in_order` and a plain `labels`
    /// keep the labels' order, `labels.any_order` lets them move.
    fn call_labels(&self, f: FnId, args: &[AArg], path: &ast::Path) -> Result<(Vec<ast::Expr>, Vec<(ast::Ident, ast::Expr)>)> {
        let name = path.spelled();
        let labels = self.items.fns[f.0 as usize].labels.clone();
        let words: Vec<String> = labels.as_ref().map(|l| l.entries.iter().map(|(_, w)| w.name.clone()).collect()).unwrap_or_default();
        let mut plain: Vec<ast::Expr> = Vec::new();
        let mut given: Vec<(ast::Ident, ast::Expr)> = Vec::new();
        for a in args {
            let AArg::Positional(e) = a else {
                return err(Diagnostic::error(format!("`{}` takes its arguments in place or with their labels, not `name = value`", name)).named("label_needs_colon").at(a.span()));
            };
            // `value:`, `count:` and `pos:` are read as the fill's words
            // until the call is known to be this function's.
            let label = match &e.kind {
                AK::Label(w, inner) => Some((w.clone(), (**inner).clone())),
                AK::Value(inner) | AK::Count(inner) | AK::Pos(inner) if labels.is_some() => {
                    let w = match &e.kind {
                        AK::Value(_) => "value",
                        AK::Count(_) => "count",
                        _ => "pos",
                    };
                    let at = Span::new(e.span.file, e.span.start as usize, e.span.start as usize + w.len());
                    Some((ast::Ident { name: w.to_string(), span: at }, (**inner).clone()))
                }
                _ => None,
            };
            match label {
                Some((w, inner)) => {
                    if !words.contains(&w.name) {
                        let has = if words.is_empty() { "none".to_string() } else { words.iter().map(|w| format!("`{}:`", w)).collect::<Vec<_>>().join(", ") };
                        return err(Diagnostic::error(format!("`{}` has no label `{}:`; its labels are {}", name, w.name, has)).named("argument_label_unknown").at(w.span).fix_close(w.span, &w.name, words.iter().map(|w| w.as_str())));
                    }
                    if let Some((prev, _)) = given.iter().find(|(p, _)| p.name == w.name) {
                        return err(Diagnostic::error(format!("`{}:` is given twice", w.name)).named("given_twice").at(w.span).why_at(prev.span, "the first is here"));
                    }
                    given.push((w, inner));
                }
                None => {
                    if let Some((prev, _)) = given.last() {
                        return err(Diagnostic::error(format!("an argument without a label comes before the labelled ones: this one comes after `{}:` (owner, 2026-10-21)", prev.name)).named("unlabelled_after_labelled").at(e.span));
                    }
                    plain.push(e.clone());
                }
            }
        }
        if let Some(l) = labels.as_ref().filter(|l| l.order != ast::LabelOrder::AnyOrder) {
            let at = |w: &str| l.entries.iter().position(|(_, x)| x.name == w).unwrap_or(0);
            for pair in given.windows(2) {
                if at(&pair[1].0.name) < at(&pair[0].0.name) {
                    return err(Diagnostic::error(format!("`{}`'s labels keep their order: `{}:` comes before `{}:` (owner, 2026-10-21)", name, pair[1].0.name, pair[0].0.name)).named("labels_out_of_order").at(pair[1].0.span));
                }
            }
        }
        Ok((plain, given))
    }

    /// One version's arguments in its parameters' order: the arguments
    /// without a label fill the parameters without one, in place, the
    /// labelled ones theirs, and a left-out one is its default.
    fn shape_args(&self, f: FnId, plain: &[ast::Expr], given: &[(ast::Ident, ast::Expr)], span: Span, path: &ast::Path) -> Result<Vec<AArg>> {
        let name = path.spelled();
        let decl = &self.items.fns[f.0 as usize];
        let word = |p: &str| decl.labels.as_ref().and_then(|l| l.entries.iter().find(|(q, _)| q.name == p)).map(|(_, w)| w.name.clone());
        let default = |p: &str| decl.defaults.iter().find(|(q, _)| q.name == p).map(|(_, v)| v.clone());
        let mut out: Vec<AArg> = Vec::new();
        let mut rest = plain.iter();
        for p in &decl.params {
            let value = match word(&p.name) {
                Some(w) => match given.iter().find(|(g, _)| g.name == w) {
                    Some((_, e)) => Some(e.clone()),
                    None => match default(&p.name) {
                        Some(d) => Some(d),
                        None => return err(Diagnostic::error(format!("`{}` needs `{}:`, its '{}': a labelled argument is written with its label (owner, 2026-10-21)", name, w, p.name)).named("argument_label_missing").at(span)),
                    },
                },
                None => rest.next().cloned().or_else(|| default(&p.name)),
            };
            // Too few: the count is told as any call's is.
            let Some(v) = value else { break };
            out.push(AArg::Positional(v));
        }
        out.extend(rest.cloned().map(AArg::Positional));
        if let Some((g, _)) = given.iter().find(|(g, _)| !decl.params.iter().any(|p| word(&p.name).as_deref() == Some(g.name.as_str()))) {
            return err(Diagnostic::error(format!("this version of `{}` takes no `{}:`", name, g.name)).named("argument_label_unknown").at(g.span));
        }
        Ok(out)
    }

    /// Which version a call takes: the one whose parameters its arguments
    /// fit exactly, widths counting (owner, 2026-10-20: "(a)"). A bare
    /// literal fits every type that holds it, so two versions it fits are
    /// two it could take, and the call is refused naming the typed literal.
    fn pick_version(&mut self, ctx: &FnCtx, cands: &[(FnId, &[AArg])], span: Span, path: &ast::Path) -> Result<FnId> {
        let name = path.spelled();
        let set: Vec<FnId> = cands.iter().map(|(f, _)| *f).collect();
        // Each version's arguments as it takes them: with defaults, one
        // version may be given more than another.
        let peeks_of: Vec<Vec<Peek>> = cands.iter().map(|(_, args)| args.iter().filter_map(|a| match a {
            AArg::Positional(e) => Some(self.peek_arg(ctx, e)),
            _ => None,
        }).collect()).collect();
        let fits = |this: &Self, k: usize| -> bool {
            let decl = &this.items.fns[set[k].0 as usize];
            let peeks = &peeks_of[k];
            decl.params.len() == peeks.len() && decl.params.iter().zip(peeks).all(|(p, k)| p.ty.as_ref().is_some_and(|t| peek_fits(k, t.base())))
        };
        let took_at: Vec<usize> = (0..set.len()).filter(|k| fits(self, *k)).collect();
        let took: Vec<FnId> = took_at.iter().map(|k| set[*k]).collect();
        let peeks: &[Peek] = took_at.first().map(|k| peeks_of[*k].as_slice()).unwrap_or(&[]);
        match took.as_slice() {
            [one] => Ok(*one),
            [] => {
                let shapes = set.iter().map(|f| {
                    let d = &self.items.fns[f.0 as usize];
                    format!("[{}]", d.params.iter().map(|p| p.ty.as_ref().map(|t| self.spell(t)).unwrap_or_default()).collect::<Vec<_>>().join(", "))
                }).collect::<Vec<_>>().join(", ");
                err(Diagnostic::error(format!("no version of `{}` takes this call: its versions take {}", name, shapes)).named("no_version_fits").at(span))
            }
            several => {
                // The bare literal the versions that take the call take
                // as different types, typed as the last of them takes it.
                let at = |f: &FnId, i: usize| self.items.fns[f.0 as usize].params.get(i).and_then(|p| p.ty.as_ref()).map(|t| t.base().clone());
                let hint = peeks.iter().enumerate().find_map(|(i, k)| match k {
                    Peek::Int(t) | Peek::Bin(t) | Peek::Deci(t) if several.iter().any(|f| at(f, i) != at(&several[0], i)) => {
                        let ty = at(&several[several.len() - 1], i)?;
                        let t = if matches!(k, Peek::Deci(_)) { format!("exactly {}", t) } else { t.clone() };
                        Some(format!("`{}: {}`", t, self.spell(&ty)))
                    }
                    _ => None,
                });
                let why = match hint {
                    Some(h) => format!("a bare literal fits more than one; say which, {}", h),
                    None => "say the arguments' types".to_string(),
                };
                err(Diagnostic::error(format!("versions of `{}` could take this call: {} (owner, 2026-10-20)", name, why)).named("call_ambiguous").at(span))
            }
        }
    }

    /// An argument's type as it is written, without checking it: what
    /// picks a version.
    fn peek_arg(&self, ctx: &FnCtx, e: &ast::Expr) -> Peek {
        match &e.kind {
            AK::Int(t) => Peek::Int(t.replace('_', "")),
            AK::Bin(t) => Peek::Bin(t.replace('_', "")),
            AK::Exact(t) => Peek::Deci(t.replace('_', "")),
            AK::Str(_) => Peek::Text,
            AK::Unary { op: UnOp::Neg, expr } => match &expr.kind {
                AK::Int(t) => Peek::Int(format!("-{}", t.replace('_', ""))),
                AK::Bin(t) => Peek::Bin(format!("-{}", t.replace('_', ""))),
                AK::Exact(t) => Peek::Deci(format!("-{}", t.replace('_', ""))),
                _ => self.peek_arg(ctx, expr),
            },
            AK::Typed { ty, .. } => self.body_type(ctx, ty).map(|t| Peek::Ty(self.resolve(&t))).unwrap_or(Peek::Unknown),
            AK::Name(q) => match ctx.lookup(&q.name) {
                Some(id) => match self.resolve(&ctx.locals[id.0 as usize].ty) {
                    t @ Ty::Int(IntW::Free(_)) | t @ Ty::Bin(BinW::Free(_)) => Peek::Free(t),
                    t => Peek::Ty(t),
                },
                None => Peek::Unknown,
            },
            AK::Verb { expr, .. } => self.peek_arg(ctx, expr),
            AK::Paren(expr) => self.peek_arg(ctx, expr),
            AK::Bool(_) => Peek::Ty(Ty::Bool),
            // A conversion's type is its target (check claude, 2026-10-20).
            AK::Conv { target, .. } => self.body_type(ctx, target).map(|t| Peek::Ty(self.resolve(&t))).unwrap_or(Peek::Unknown),
            // Arithmetic is its typed operand's type (std claude, 2026-10-20:
            // `0.0 - 'y'`); a comparison, a call or a field gives none, as
            // in the oracle (c8a27e08).
            AK::Binary { op, lhs, rhs, .. } => {
                if !matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::Mod(_)) {
                    return Peek::Unknown;
                }
                match (self.peek_arg(ctx, lhs), self.peek_arg(ctx, rhs)) {
                    (Peek::Ty(t), _) | (_, Peek::Ty(t)) => Peek::Ty(t),
                    (Peek::Free(t), _) | (_, Peek::Free(t)) => Peek::Free(t),
                    _ => Peek::Unknown,
                }
            }
            _ => Peek::Unknown,
        }
    }

    pub(crate) fn fn_call_one(&mut self, ctx: &mut FnCtx, id: FnId, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool, path: &ast::Path) -> Result<Expr> {
        let decl = self.items.fns[id.0 as usize].clone();
        if decl.op.is_some() {
            return err(Diagnostic::error(format!("`{}` is an operator; it goes between two values: `$a {} $b`, never `{}(…)`", decl.name, decl.name, decl.name)).named("operator_called_as_function").at(span));
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
        // std's `str.at.X` and `str.slice.X`, written in CyborgPL, say each
        // position with `pos` as the built-in ones do (owner, 2026-10-08:
        // "pos on every position, list.remove and str.at / str.slice too"):
        // checked here, and handed on as the number.
        let unpos: Vec<ast::Arg>;
        let sliced: Vec<ast::Arg>;
        let args = if self.items.nerve_cord.is_some_and(|c| self.items.kit_root(decl.module) == c) && (decl.path.starts_with("std::str.at.") || decl.path.starts_with("std::str.slice.")) {
            let short = decl.path.trim_start_matches("std::").to_string();
            // A slice says where it runs, `from pos 2, to pos 4` or `before
            // pos 5`, as the built-in one does (std claude, 2026-10-20).
            let args = if decl.path.starts_with("std::str.slice.") {
                sliced = slice_args(&format!("std::{}", short), args, span)?;
                &sliced[..]
            } else {
                args
            };
            let mut out = Vec::new();
            for (k, a) in args.iter().enumerate() {
                out.push(match a {
                    ast::Arg::Positional(e) if k >= 1 => match &e.kind {
                        AK::Pos(inner) if written_below_one(inner) => return err(Diagnostic::error(NO_POSITION).named("index_out_of_range").at(inner.span)),
                        AK::Pos(inner) => ast::Arg::Positional((**inner).clone()),
                        _ => return err(Diagnostic::error(format!("a position says `pos`: `std::{}(lend $s, pos: 3)`", short)).named("position_unlabelled").at(e.span).fix_edit(e.span, Edit::Label, "pos: ")),
                    },
                    other => other.clone(),
                });
            }
            unpos = out;
            &unpos[..]
        } else {
            args
        };
        // Unpack the arguments: spreads count as their length.
        let flat = self.flatten_args(ctx, args, span)?;
        if !has_variadic && flat.len() != fixed {
            return err(Diagnostic::error(format!("`{}` takes {} argument{}, and {} {} given", decl.name, fixed, if fixed == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).named("wrong_argument_count").at(span).why_at(decl.name_span, format!("`{}` is declared here", decl.name)));
        }
        if has_variadic && flat.len() < fixed {
            return err(Diagnostic::error(format!("`{}` takes at least {} argument{}, and {} {} given", decl.name, fixed, if fixed == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).named("wrong_argument_count").at(span));
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
                    return err(Diagnostic::error(format!("one blank, one type: `<{}>` is filled by an earlier argument as `{}`, and '{}' would make it something else", blank, self.spell(&pty), pname)).named("generic_filled_inconsistently").at(e.span.unwrap_or(span)));
                }
                Err(e) => return Err(e),
            };
            if variadic {
                // A typed variadic holds values of its type, whatever each
                // argument's own width (a bare number's is none): `...'nums':
                // i32` is a list of i32, not of the exact type.
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
                return err(Diagnostic::error(format!("the blank `<{}>` of `{}` is not filled by this call: no argument says what it is", decl.blanks[i].0, decl.name)).named("generic_unfilled").at(span));
            }
            // interface bound
            for iface in decl.blanks[i].1.clone() {
                if let Err(reason) = self.satisfies(&r, iface) {
                    let iname = self.items.ifaces[iface.0 as usize].name.clone();
                    return err(Diagnostic::error(format!("`{}` is not `{}`: {} — `{}` asks for `<{}>: {}`", self.spell(&r), iname, reason, decl.name, decl.blanks[i].0, iname)).named("bound_not_satisfied").at(span));
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
        // Named as the call writes it: `std::str.at.grapheme`, `area.square`.
        let ow = self.call_otherwise(ctx, otherwise, &sig.fails, &sig.ret, used, span, &path.spelled())?;
        // A borrow handed back is held by a name, handed straight to a
        // parameter or a print, or handed back again by a function that
        // says so (owner, 2026-09-29) — anywhere else it would be a second
        // owner of what it borrows.
        if self.items.fns[id.0 as usize].ret_lent.is_some() && ctx.lend_call_ok != Some(span) {
            return err(Diagnostic::error(format!("`{}` hands back a borrow, which is held by a name (`let $l := {}(…);`) or handed straight to a parameter: here it would be a second owner of what it borrows", self.items.fns[id.0 as usize].name, self.items.fns[id.0 as usize].name)).named("returned_borrow_misused").at(span));
        }
        self.lends_apart(ctx, &hargs, span)?;
        Ok(Expr { kind: K::Call { callee: Callee::Fn(inst), args: hargs, otherwise: ow, fails: sig.fails.clone() }, ty: sig.ret, span })
    }

    fn call_otherwise(&mut self, ctx: &mut FnCtx, otherwise: Option<&ast::Otherwise>, fails: &[(String, Option<Ty>)], ret: &Ty, used: bool, span: Span, name: &str) -> Result<Option<Box<hir::Otherwise>>> {
        match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error(format!("`{}` cannot fail; drop the `otherwise`", name)).named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                Ok(Some(Box::new(self.otherwise(ctx, o, fails, ret, used, span)?)))
            }
            None => {
                // `must.fail` catches the reasons itself (owner, 2026-09-29).
                if !fails.is_empty() && !ctx.must_fail {
                    let list = fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", ");
                    return err(Diagnostic::error(format!("`{}` can fail ({}): say what happens — `otherwise` a value, `otherwise return …`, `otherwise fail`, or `otherwise {{ {} {{ … }} }}` arms", name, list, fails[0].0)).named("failure_unhandled").at(span));
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
                // `cord::exit[…]` leaves: no value is given.
                if matches!(&v.kind, K::Call { callee: Callee::Builtin(Builtin::Std(n)), .. } if n == "exit") {
                    let span = v.span;
                    return Ok(hir::Otherwise::Leave(Box::new(hir::Stmt { kind: hir::StmtKind::Expr(v), span, tier: ctx.tier })));
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
                    return err(Diagnostic::error("`MAIN` cannot fail, so it cannot pass a failure up: handle it here, or exit").named("main_cannot_fail").at(*sp));
                }
                for (n, _) in fails {
                    if !ctx.fails.iter().any(|(x, _)| x == n) {
                        if ctx.fails.is_empty() {
                            return err(Diagnostic::error(format!("`{}` cannot fail, so it cannot pass `{}` up: add `-> … or fail[{}]` to its signature, or handle it here", ctx.name, n, n)).named("failure_not_listed").at(*sp));
                        }
                        return err(Diagnostic::error(format!("`{}` does not list `{}` among its failures, so `otherwise fail` cannot pass it up: its signature says fail[{}] — add `{}` there, or handle it here", ctx.name, n, ctx.fails.iter().map(|(x, _)| x.clone()).collect::<Vec<_>>().join(", "), n)).named("failure_not_listed").at(*sp));
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
                        return err(Diagnostic::error(format!("this call never fails with `{}`: it fails with {}", arm.name.name, fails.iter().map(|(n, _)| format!("`{}`", n)).collect::<Vec<_>>().join(", "))).named("failure_never_happens").at(arm.name.span));
                    }
                }
                let missing: Vec<&str> = fails.iter().enumerate().filter(|(i, _)| !covered[*i]).map(|(_, (n, _))| n.as_str()).collect();
                if !missing.is_empty() {
                    return err(Diagnostic::error(format!("this call can also fail with {}: every reason needs an arm", missing.iter().map(|m| format!("`{}`", m)).collect::<Vec<_>>().join(", "))).named("failure_case_unhandled").at(*arms_span));
                }
                let mut covered = vec![false; fails.len()];
                for arm in arms {
                    let Some(fi) = fails.iter().position(|(n, _)| *n == arm.name.name) else {
                        return err(Diagnostic::error(format!("this call never fails with `{}`: it fails with {}", arm.name.name, fails.iter().map(|(n, _)| format!("`{}`", n)).collect::<Vec<_>>().join(", "))).named("failure_never_happens").at(arm.name.span));
                    };
                    if covered[fi] {
                        return err(Diagnostic::error(format!("`{}` is handled twice", arm.name.name)).named("failure_handled_twice").at(arm.name.span));
                    }
                    covered[fi] = true;
                    ctx.push_scope();
                    let bind = match (&arm.bind, &fails[fi].1) {
                        (None, None) => None,
                        (Some(b), Some(t)) => Some(ctx.declare(&b.name, Local { name: b.name.clone(), ty: t.clone(), mutability: None, kind: LocalKind::Owned, from_blank: false, span: b.span })),
                        (Some(b), None) => return err(Diagnostic::error(format!("`{}` carries nothing, so there is no value to name '{}'", arm.name.name, b.name)).named("carries_nothing").at(b.span)),
                        (None, Some(t)) => return err(Diagnostic::error(format!("`{}` carries {}: name it — `{} $value {{ … }}`", arm.name.name, crate::types::a_type(&self.spell(t)), arm.name.name)).named("data_missing").at(arm.name.span)),
                    };
                    // An arm gives the call's value, as a `match` case does
                    // (owner, 2026-09-29), or leaves.
                    let (body, given) = self.arm_block(ctx, &arm.body)?;
                    ctx.pop_scope();
                    let given = match given {
                        Some(_) if !used => return err(Diagnostic::error(format!("the `{}` arm gives a value, but the call's value is not used", arm.name.name)).named("handler_value_unused").at(arm.body.span)),
                        Some(g) => Some(self.expect_store(ctx, g, ret, "the call's value")?),
                        None => None,
                    };
                    if used && given.is_none() && !super::stmt::leaves_block(&body) {
                        return err(Diagnostic::error(format!("the `{}` arm must give the call's value (`{{ 0 }}`) or leave (`return`, `fail`, `break`, `continue` or `std::exit`)", arm.name.name)).named("handler_falls_through").at(arm.body.span));
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
                AArg::Named(n, _) => return err(Diagnostic::error(format!("`{} = …` names a field, but this is a call: arguments are values in order", n.name)).named("field_name_in_call").at(n.span)),
                AArg::Pair(k, _) => return err(Diagnostic::error("`key = value` pairs belong in a `map[…]` literal, not a call").named("field_name_in_call").at(k.span)),
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
                    None => return err(Diagnostic::error(format!("the length of {} is not known here, so `...` cannot unpack it: a call's shape is fixed per call — give the list a bound (`list[T][0, 10]`) or pass it as a list", self.place_name(ctx, &place))).named("spread_length_unknown").at(span).why_at(local.span, "declared here with no bound")),
                }
            }
            other => return err(Diagnostic::error(format!("`...` unpacks a list, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(span)),
        };
        let elem = self.elem_type_of(&t, span, "unpacking")?;
        let elem = self.resolve(&elem);
        let elem_owned = elem.is_owned() && !matches!(elem, Ty::Bundle(_));
        let verb = match verb {
            Some(v) if !elem_owned => return err(Diagnostic::error(format!("`...{} {}`: its elements are plain `{}` values, which copy by themselves — write `...{}`", v.spelled(), self.place_name(ctx, &place), self.spell(&elem), self.place_name(ctx, &place))).named("spread_verb_wrong").at(span).fix_edit(span, Edit::Swap(vec![format!("...{} ", v.spelled())]), "...")),
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
                    return err(Diagnostic::error(format!("`...` unpacks into a variadic parameter, and `{}` has none: the values would land on a fixed parameter", fname)).named("spread_not_variadic").at(span));
                }
                // Each element must fit the variadic's element type.
                if !self.unify(&elem, pty) && !matches!(elem, Ty::Bundle(_)) {
                    return err(Diagnostic::error(format!("`{}` takes `{}` values, and the elements of {} are `{}`", fname, self.spell(pty), self.place_name(ctx, &place), self.spell(&elem))).named("wrong_type").at(span));
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
                                Some(v) => return err(Diagnostic::error(format!("`...{}` cannot hand elements to an owning parameter: say `...copy` (each duplicated) or `...give` (each moved out, the list left empty)", v.spelled())).named("spread_verb_wrong").at(span).fix_edit(span, Edit::Swap(vec![format!("...{}", v.spelled())]), "...copy").fix_edit(span, Edit::Swap(vec![format!("...{}", v.spelled())]), "...give")),
                                None => return err(Diagnostic::error(format!("the elements of {} are owned `{}` values and `{}` takes ownership of each: say `...copy {}` or `...give {}`", self.place_name(ctx, &place), self.spell(&elem), fname, self.place_name(ctx, &place), self.place_name(ctx, &place))).named("owned_value_bare").at(span)),
                            }
                        } else {
                            Arg::Lend { place, mutable: false }
                        }
                    }
                    ParamKind::Lent | ParamKind::LentMut => {
                        if let Some(v) = verb {
                            return err(Diagnostic::error(format!("`{}` takes `lent ...{}`: `...` with no verb lends each element — drop the `{}`", fname, pname, v.spelled())).named("spread_verb_wrong").at(span).fix_edit(span, Edit::Swap(vec![format!("...{} ", v.spelled())]), "..."));
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
                    // As the oracle says it.
                    (ParamKind::Lent, Verb::Give | Verb::Copy) => return err(Diagnostic::error(format!("`{}` only looks at '{}': it is `lent`, so the caller writes `lend`, not `{}`", fname, pname, verb.spelled())).named("verb_wrong_for_parameter").at(e.span).fix_edit(e.span, Edit::Verb, "lend")),
                    (ParamKind::Lent, Verb::LendMut) => return err(Diagnostic::error(format!("`{}` only looks at '{}' (it is `lent`): `lend.mut` offers more than it takes — write `lend`", fname, pname)).named("verb_wrong_for_parameter").at(e.span).fix_edit(e.span, Edit::Verb, "lend")),
                    (ParamKind::LentMut, Verb::Lend) => return err(Diagnostic::error(format!("`{}` may modify '{}' (it is `lent.mut`): write `lend.mut`", fname, pname)).named("lend_mut_needed").at(e.span).fix_edit(e.span, Edit::Verb, "lend.mut")),
                    (ParamKind::LentMut, Verb::Give | Verb::Copy) => return err(Diagnostic::error(format!("`{}` may modify '{}' (it is `lent.mut`): write `lend.mut`, not `{}`", fname, pname, verb.spelled())).named("lend_mut_needed").at(e.span).fix_edit(e.span, Edit::Verb, "lend.mut")),
                    (ParamKind::Owned, Verb::Lend | Verb::LendMut) => {
                        let whom = if pname.starts_with("the ") { pname.to_string() } else { format!("'{}'", pname) };
                        return err(Diagnostic::error(format!("`{}` takes ownership of {}, and you lent it: write `give` (it moves) or `copy` (a second one)", fname, whom)).named(if fname == "channel.send" { "mcps::send_needs_give" } else { "verb_wrong_for_parameter" }).at(e.span).fix_edit(e.span, Edit::Verb, "give").fix_edit(e.span, Edit::Verb, "copy"));
                    }
                    _ => {}
                }
                let place = self.place(ctx, expr, use_)?;
                let pt = self.resolve(&place.ty);
                let name = self.place_name(ctx, &place);
                let fb = ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty();
                self.check_verb_on_type(&pt, *verb, fb, e.span, &name)?;
                if !self.unify(&pt, pty) {
                    return err(Diagnostic::error(format!("{} is {}, and {} is {}", what, crate::types::a_type(&self.spell(pty)), name, crate::types::a_type(&self.spell(&pt)))).named(if matches!((self.resolve(pty).base(), pt.base()), (Ty::Seq { kind: SeqKind::List, .. }, Ty::Seq { kind: SeqKind::Array, .. }) | (Ty::Seq { kind: SeqKind::Array, .. }, Ty::Seq { kind: SeqKind::List, .. })) { "list_array_mismatch" } else { "wrong_type" }).at(e.span));
                }
                // A name whose numbers' width was never written (`:=` of a
                // list of bare numbers) takes it from here: Precog picks the
                // type from use (`types.md`), and a lend hands over the
                // value where it is, so the callee's width is the one it has
                // to be. Left at the exact width, `total[lend 'xs']` on a
                // `list[i32]` read the wrong layout.
                if place.proj.is_empty() {
                    let pty_r = self.resolve(pty);
                    if let Some(adopted) = adopt_widths(&pt, &pty_r) {
                        ctx.locals[place.base.0 as usize].ty = adopted;
                    }
                }
                // "A lend hands over the value where it is, so the sizes
                // must match" (owner, 2026-09-24: a; `ownership-syntax.md`):
                // a `list[i64]` lent to a `list[i32]`. Bounds change no
                // layout; a width not written and a blank `<T>` are not
                // compared.
                if matches!(verb, Verb::Lend | Verb::LendMut) {
                    let have = self.resolve(&ctx.locals[place.base.0 as usize].ty);
                    let have = if place.proj.is_empty() { have } else { pt.clone() };
                    let want = self.resolve(pty);
                    if !sizes_match(&have, &want) {
                        return err(Diagnostic::error(format!("{} is {}, and {} is {}: a lend hands over the value where it is, so the sizes must match", what, crate::types::a_type(&self.spell(&want)), name, crate::types::a_type(&self.spell(&have)))).named("lend_size_mismatch").at(e.span));
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
                self.want = Some(pty.clone());
                let was_plain = self.plain_numbers;
                self.plain_numbers = plain_number(&self.resolve(pty));
                let v = self.expr(ctx, e);
                self.plain_numbers = was_plain;
                self.want = None;
                ctx.lend_call_ok = saved;
                let v = v?;
                // A borrow a call hands back, used where it stands, goes to a
                // parameter as a `lend` (or `lend.mut`) written there would.
                if let (Some(mutable), K::Call { callee: Callee::Fn(inst), .. }) = (self.call_lends(&v), &v.kind) {
                    let callee = self.items.fns[self.instances[inst.0 as usize].key.fn_id.0 as usize].name.clone();
                    match (kind, mutable) {
                        (ParamKind::Owned, _) => return err(Diagnostic::error(format!("`{}` takes ownership of '{}', and `{}` hands back a borrow: hold it by a name and `copy` it, or pass what it borrows", fname, pname, callee)).named("verb_wrong_for_parameter").at(e.span)),
                        (ParamKind::Lent, true) => return err(Diagnostic::error(format!("`{}` only looks at '{}' (it is `lent`): `{}` hands back a `lent.mut` borrow, which offers more than it takes", fname, pname, callee)).named("verb_wrong_for_parameter").at(e.span)),
                        (ParamKind::LentMut, false) => return err(Diagnostic::error(format!("`{}` may modify '{}' (it is `lent.mut`): `{}` hands back a `lent` borrow, which only looks", fname, pname, callee)).named("lend_mut_needed").at(e.span)),
                        _ => {}
                    }
                }
                let vt = self.resolve(&v.ty);
                if let K::Read(place) = &v.kind {
                    if vt.is_owned() && !(ctx.locals[place.base.0 as usize].from_blank && place.proj.is_empty()) {
                        let name = self.place_name(ctx, place);
                        return match kind {
                            ParamKind::Owned => err(Diagnostic::error(format!("{} takes ownership: say what happens to {} — `give {}` (it moves) or `copy {}` (a second one)", what, name, name, name)).named("owned_value_bare").at(e.span).fix_edit(e.span, Edit::Verb, "give").fix_edit(e.span, Edit::Verb, "copy")),
                            // As the oracle says it (c322239b): the function,
                            // what it does with the value, and the verb.
                            ParamKind::Lent => err(Diagnostic::error(format!("`{}` only looks at {} — pass `lend {}`", fname, name, name)).named("owned_value_bare").at(e.span).fix_edit(e.span, Edit::Verb, "lend")),
                            ParamKind::LentMut => err(Diagnostic::error(format!("`{}` may modify {} — pass `lend.mut {}`", fname, name, name)).named("lend_mut_needed").at(e.span).fix_edit(e.span, Edit::Verb, "lend.mut")),
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
                        return err(Diagnostic::error(format!("`{}` needs to know which width it works in, and a generic parameter gives none: give an operand a type (`to.u8(0) minus.wrap 1`)", op)).named("width_unknown").at(e.span));
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
            // `pos` on every index (owner, 2026-10-07).
            Ty::Seq { .. } if args.len() == 1 => return err(Diagnostic::error(format!("a read says it is a position: `{}[pos: 2]`", self.expr_name(ctx, &c))).named("position_unlabelled").at(span).fix_edit(args[0].span(), Edit::Label, "pos: ")),
            Ty::Seq { .. } if otherwise.is_some() => return err(Diagnostic::error(format!("reading {}[…] cannot fail — a position past the end is a check site — so it takes no `otherwise`", self.expr_name(ctx, &c))).named("otherwise_never_needed").at(span)),
            Ty::Seq { .. } => return err(Diagnostic::error(format!("{} is a list, and a list is read one position at a time: `{}[pos: 2]`", self.expr_name(ctx, &c), self.expr_name(ctx, &c))).named("parens_for_brackets").at(span)),
            other => return err(Diagnostic::error(format!("{} is {}, not a function: `$f(…)` calls a held function, and `$xs[pos: 2]` reads a list", self.expr_name(ctx, &c), crate::types::a_type(&self.spell(other)))).named("not_a_function").at(span)),
        };
        let flat = self.flatten_args(ctx, args, span)?;
        if flat.len() != ft.params.len() {
            return err(Diagnostic::error(format!("this function takes {} argument{}, and {} {} given", ft.params.len(), if ft.params.len() == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).named("wrong_argument_count").at(span));
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
        let name = self.items.externs[id.0 as usize].name.clone();
        self.power_use(ctx, &["c_calls", "functions"], Some(name.clone()), format!("calls C (`{}`)", name), span);
        let info = self.items.externs[id.0 as usize].clone();
        if info.is_let {
            return err(Diagnostic::error(format!("'{}' is C's variable, not a function", info.name)).named("not_a_function").at(span));
        }
        if self.needs_mark(ctx) {
            self.power_first()?;
            return err(Diagnostic::error(format!("a call into C is a no_check site: nothing about `{}` can be proven — write `no_check` in front of the statement", info.name)).named("cnc::no_check_missing").at(span).fix_edit(span, crate::diag::Edit::Before, "no_check "));
        }
        let flat = self.flatten_args(ctx, args, span)?;
        // A fixed shape takes what it names, no more and no fewer (owner,
        // 2026-10-19).
        if !info.variadic && info.var_from.is_some() && flat.len() != info.params.len() {
            return err(Diagnostic::error(format!("`{}` takes exactly {} argument(s), as its shape says, and {} {} given", info.name, info.params.len(), flat.len(), if flat.len() == 1 { "was" } else { "were" })).named("wrong_argument_count").at(span));
        }
        if !info.variadic && flat.len() != info.params.len() {
            return err(Diagnostic::error(format!("`{}` takes {} argument{}, and {} {} given", info.name, info.params.len(), if info.params.len() == 1 { "" } else { "s" }, flat.len(), if flat.len() == 1 { "was" } else { "were" })).named("wrong_argument_count").at(span));
        }
        if info.variadic && flat.len() < info.params.len() {
            return err(Diagnostic::error(format!("`{}` takes at least {} argument{}", info.name, info.params.len(), if info.params.len() == 1 { "" } else { "s" })).named("wrong_argument_count").at(span));
        }
        let mut hargs = Vec::new();
        for (i, fa) in flat.into_iter().enumerate() {
            let FlatArg::One(e) = fa else {
                return err(Diagnostic::error("`...` cannot unpack into a C call").named("cnc::spread_into_c").at(span));
            };
            if i < info.params.len() {
                let (pname, pty) = info.params[i].clone();
                hargs.push(self.c_argument(ctx, &e, &pty, &info.name, &pname)?);
            } else if matches!(e.kind, AK::Verb { verb: Verb::Lend | Verb::LendMut, .. }) {
                // A lent list or text among C's extras is a pointer to its
                // first element (owner, 2026-10-19).
                let to = Ty::Ptr { elem: Box::new(Ty::int(false, 8)), mutable: matches!(e.kind, AK::Verb { verb: Verb::LendMut, .. }) };
                hargs.push(self.c_argument(ctx, &e, &to, &info.name, &format!("{}", i + 1))?);
            } else {
                // C's own varargs: each extra argument must be C-shaped.
                let v = self.expr(ctx, &e)?;
                let vt = self.resolve(&v.ty);
                if !vt.base().is_c_shaped(&self.items.structs, &self.items.enums) && !matches!(vt.base(), Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_))) {
                    return err(Diagnostic::error(format!("{} cannot cross to C's varargs: only C-shaped values can", crate::types::a_type(&self.spell(&vt)))).named("cnc::cannot_cross_to_c").at(e.span));
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
                    return err(Diagnostic::error(format!("an `extern fn` cannot fail; drop the `otherwise` after `{}`", info.name)).named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                Some(Box::new(self.otherwise(ctx, o, &fails, &info.ret, used, span)?))
            }
            None => {
                if !fails.is_empty() && !ctx.must_fail {
                    let what = if info.ret.bound().is_some() { "its return is a range C promised" } else { "what C hands back is checked at the door — a C enum's number must be one of its cases, a ranged field in its range" };
                    let what = if info.ret.bound().is_some() { format!("{}, checked at the door", what) } else { what.to_string() };
                    return err(Diagnostic::error(format!("`{}` can fail (out_of_range): {} — say what happens with `otherwise`", info.name, what)).named("failure_unhandled").at(span));
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
                        return err(Diagnostic::error(format!("`{}`'s parameter '{}' is a `ptr.mut`: C may write through it, so write `lend.mut`", fname, pname)).named("lend_mut_needed").at(e.span).fix_edit(e.span, Edit::Verb, "lend.mut"));
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
                AK::Verb { verb, .. } => return err(Diagnostic::error(format!("`{}` hands C a pointer: `lend` (or `lend.mut`) becomes the address; `{}` does not", fname, verb.spelled())).named("verb_wrong_for_parameter").at(e.span)),
                _ => {
                    // a held pointer, or a capture-free function for a callback slot
                    let v = self.expr(ctx, e)?;
                    let vt = self.resolve(&v.ty);
                    if !self.unify(&vt, pty) {
                        // A pointer to another target than C's (owner,
                        // 2026-10-10: `FILE` is not `u8`), with the way
                        // to say it is meant.
                        if let Ty::Ptr { elem: got, .. } = vt.base() {
                            let (want, got) = (self.spell(elem), self.spell(got));
                            return err(Diagnostic::error(format!("`{}` wants a pointer to `{}` for '{}', and this points at `{}` (`ptr.cast[{}][…]` if you mean it)", fname, want, pname, got, want)).named("cnc::pointer_target_wrong").at(e.span));
                        }
                        return err(Diagnostic::error(format!("`{}`'s parameter '{}' is {}, and this is {}", fname, pname, crate::types::a_type(&self.spell(pty)), crate::types::a_type(&self.spell(&vt)))).named("wrong_type").at(e.span));
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
                    return err(Diagnostic::error("a function handed to C cannot capture anything: C has nowhere to keep them — a C callback slot holds an address and nothing else".to_string()).named("cnc::callback_captures").at(e.span));
                }
            }
            if let Ty::Fn(got) = vt.base() {
                if got.params.len() != ft.params.len() {
                    return err(Diagnostic::error(format!("`{}`'s parameter '{}' wants a function taking {} argument(s); got a function taking {} argument(s)", fname, pname, ft.params.len(), got.params.len())).named("function_shape_wrong").at(e.span));
                }
            }
            if !self.unify(&vt, pty) {
                return err(Diagnostic::error(format!("`{}`'s parameter '{}' is {}, and this is {}", fname, pname, crate::types::a_type(&self.spell(pty)), crate::types::a_type(&self.spell(&vt)))).named("wrong_type").at(e.span));
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
            return err(Diagnostic::error(format!("a lend of {} (a `{}`) does not give C a `ptr[{}]`", self.place_name(ctx, place), self.spell(&pt), self.spell(elem))).named("cnc::pointer_target_wrong").at(span));
        }
        Ok(())
    }

    fn fn_literal(&mut self, ctx: &mut FnCtx, lit: &ast::FnLit, span: Span, want: Option<&Ty>) -> Result<Expr> {
        // A short lambda's types are those of where it goes (owner,
        // 2026-10-24: "(c)").
        let from = if lit.short {
            let ft = match want.map(|t| self.resolve(t)) {
                Some(Ty::Fn(ft)) => *ft,
                _ => return err(Diagnostic::error("a short lambda takes its types from where it goes, and here nothing says them: `let $f: fn[i32] -> bool = fn [$s] = …;` (owner, 2026-10-24)").named("lambda_types_unknown").at(span)),
            };
            if ft.params.len() != lit.params.len() {
                return err(Diagnostic::error(format!("this lambda takes {} value{}, and where it goes takes {}", lit.params.len(), if lit.params.len() == 1 { "" } else { "s" }, ft.params.len())).named("function_shape_wrong").at(span));
            }
            Some(ft)
        } else {
            None
        };
        let fn_id = self.declare_literal(lit, ctx.module, span, from.as_ref())?;
        // A function written inside one keeps its variables' case.
        if ctx.fold_vars {
            self.items.case_vars.insert(span);
        }
        // Captures: resolved in the enclosing scope, each with its verb.
        let mut caps = Vec::new();
        let mut inst_caps = Vec::new();
        for c in &lit.captures {
            let Some(outer) = ctx.lookup(&c.name.name) else {
                return Err(ctx.not_declared(Diagnostic::error(format!("'{}' is not declared", c.name.name)).named("not_declared").at(c.name.span), &c.name.name, c.name.span));
            };
            let ty = self.resolve(&ctx.locals[outer.0 as usize].ty);
            if lit.params.iter().any(|p| p.name.name == c.name.name) {
                return err(Diagnostic::error(format!("'{}' is both a capture and a parameter: they cannot share a name", c.name.name)).named("already_declared").at(c.name.span));
            }
            if c.verb == Verb::LendMut {
                ctx.changed.insert(outer);
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
            return err(Diagnostic::error(format!("`op {}` cannot skip anything: a function's arguments are worked out before it runs — write `op {}`", spelled, plain)).named("operator_cannot_be_value").at(span));
        }
        let arity = if unary.is_some() { 1 } else { 2 };
        if ft.params.len() != arity {
            if *op == B::Sub && ft.params.len() == 1 {
                return err(Diagnostic::error("`op -` subtracts; negation as a value is `op neg`").named("old_spelling").at(span).fix_edit(span, Edit::Replace, "op neg"));
            }
            return err(Diagnostic::error(format!("`op {}` takes {} operand{}, and this wants a function of {}", spelled, arity, if arity == 1 { "" } else { "s" }, ft.params.len())).named("function_shape_wrong").at(span));
        }
        if !ft.fails.is_empty() {
            return err(Diagnostic::error(format!("`op {}` cannot fail, and this wants a function that can", spelled)).named("function_shape_wrong").at(span));
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
        self.items.fns.push(crate::resolve::FnDeclInfo { name: format!("op {}", spelled), path: String::new(), module: ctx.module, vis: crate::ast::Vis::Private, params, ret: ft.ret.clone(), ret_lent: None, fails: Vec::new(), blanks: Vec::new(), body: Some(body), op: None, is_literal: true, test: false, table_bytes: None, labels: None, defaults: Vec::new(), span, name_span: span });
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
        ctx.fold_vars = self.items.case_vars.contains(&decl.name_span);
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
            if p.mutability == Some(ast::Mutability::Mut) {
                ctx.promised.push(id);
            }
            params.push(id);
        }
        let block = self.block(&mut ctx, &body)?;
        if sig.ret != Ty::Nothing && !super::stmt::leaves_block(&block) {
            return err(Diagnostic::error(format!("this function returns {} but can fall off the end: every path must `return` a value", crate::types::a_type(&self.spell(&sig.ret)))).named("missing_return").at(decl.span));
        }
        self.mut_promises_kept(&ctx)?;
        let mut block = block;
        self.const_views_first(&ctx, &mut block);
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
                    return err(Diagnostic::error(format!("'{}' is lent `lend.mut` and lent again in the same call: while something is lent to change, nothing else may borrow it", name)).named("lent_twice_in_call").at(span).why_at(*at, "lent to change here"));
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
            _ => format!("{}", crate::types::a_type(&self.spell(t))),
        }
    }

    fn conversion(&mut self, ctx: &mut FnCtx, target: &ast::Type, operand: &ast::Expr, otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        // `to.useful.d64[…]`: a bin's shortest decimal (owner,
        // 2026-09-29), where `to.d64[…]` is its exact value; and
        // `to.useful.str.utf8[…]`, the useful text, where `to.str.utf8` is
        // the exact one. `.useful` follows the verb (owner, 2026-09-30).
        // `to.stored.` / `to.useful.` (owner, 2026-10-02): where a value has
        // two readings — a number, a `bool` or a case as text, a bin as a
        // deci — the conversion says which.
        let (form, stripped) = conv_form(target)?;
        let target = stripped.as_ref().unwrap_or(target);
        // `to.d32.written[…]` (owner, 2026-10-09: "yea, `.written`"):
        // text's digits counted as written, trailing zeros included.
        let mut written = false;
        let unwritten;
        let target = match &target.kind {
            ast::TypeKind::Path { path, args } if path.segs.len() >= 2 && path.segs.last().is_some_and(|s| s.name == "written") => {
                let mut p = path.clone();
                p.segs.pop();
                written = true;
                unwritten = ast::Type { kind: ast::TypeKind::Path { path: p, args: args.clone() }, span: target.span };
                &unwritten
            }
            _ => target,
        };
        // `to.i8.bits[…]` (owner, 2026-10-06): the raw form, the Brainstem's since 2026-10-08.
        if let ast::TypeKind::Path { path, args } = &target.kind {
            if path.segs.len() >= 2 && path.segs.last().is_some_and(|s| s.name == "bits") {
                let mut p = path.clone();
                p.segs.pop();
                let inner = ast::Type { kind: ast::TypeKind::Path { path: p, args: args.clone() }, span: target.span };
                return self.to_bits(ctx, &inner, operand, otherwise, span);
            }
        }
        let t = self.body_type(ctx, target)?;
        if let Some(u) = form {
            if !matches!(t.base(), Ty::Str(_) | Ty::Deci(_)) {
                let f = if u { "useful" } else { "stored" };
                return err(Diagnostic::error(format!("`to.{}` makes text or a deci: `to.{}.str.utf8[…]`, `to.{}.d64[…]`; `to.{}` has one form", f, f, f, self.spell(&t))).named("no_such_conversion").at(span));
            }
        }
        let mut useful = form == Some(true);
        let mut v = self.expr(ctx, operand)?;
        // A conversion is not context (`operators.md`): a shift inside it
        // still waiting for its width has none.
        if self.width_pending(&self.resolve(&v.ty)) {
            let t = self.resolve(&v.ty);
            let what = self.widths_pending.iter().find(|(p, _, _)| self.resolve(p) == t).map(|(_, at, w)| (w.clone(), *at));
            if let Some((w, at)) = what {
                return err(Diagnostic::error(format!("`{}` needs a width, and nothing here gives one: a conversion takes its value as it is, so the shifted number needs a type (`to.i64(…) shr …`)", w)).named("width_unknown").at(at));
            }
        }
        // Bare numbers alone are worked out exactly, whatever is between
        // them: `to.u64[0 / ((2 / 10047273927188643194) + 1)]` divides by
        // 1. Held at `i64`, the big one read as negative (seed 371).
        if literal_only(&v) && matches!(self.resolve(&v.ty).base(), Ty::Int(IntW::Free(_))) && matches!(t.base(), Ty::Int(_)) {
            let to = exact_for(&v);
            retype_literal_arith(&mut v, &to);
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
            // A brain16 widens to an f64 exactly, and a deci reaches a
            // brain16 through one: the bin's conversions, failures and all.
            (Ty::Brain, Ty::Deci(_)) => fails.extend(crate::deci::conv_fails(&Ty::Bin(BinW::Fixed(64)), &t, None).unwrap_or_default()),
            (Ty::Deci(_), Ty::Brain) => fails.extend(crate::deci::conv_fails(&vt, &Ty::Bin(BinW::Fixed(64)), None).unwrap_or_default()),
            (Ty::Int(_) | Ty::Bin(_) | Ty::Str(_) | Ty::Deci(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Int(_) | Ty::Bin(_) | Ty::Str(_)) => {
                // A `:=` whole number has the digits its operands allow
                // (owner, 2026-10-12: "(a)"); a typed one, its type's.
                let free = matches!(vt.base(), Ty::Int(IntW::Free(_))) || vt.is_exact();
                let literal = match &v.kind {
                    K::Int(i) => Some(i.unsigned_abs().to_string().len() as u32),
                    K::BigInt(d) => Some(d.trim_start_matches('-').len() as u32),
                    _ if free => self.int_span(ctx, &v).map(|(lo, hi)| lo.unsigned_abs().max(hi.unsigned_abs()).to_string().len() as u32),
                    _ => None,
                };
                fails.extend(crate::deci::conv_fails(&vt, &t, literal).unwrap_or_default());
            }
            (Ty::Int(_), Ty::Int(_)) => {}
            (Ty::Int(_), Ty::Bin(_)) => {}
            // The brain float (owner, 2026-10-02): into it rounds, from a
            // number or from text; out of it, an f32 or f64 is exact,
            // an f16 a check site past its largest, an integer as from a bin.
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
                    return err(Diagnostic::error(format!("a `{}` is not text: its case `{}` carries data, so no `{}` converts, whichever case it holds; `match` it and build the text you want, or print it directly (owner, 2026-09-30)", info.name, c.name, info.name)).named("not_text").at(span));
                }
            }
            (Ty::Struct { id, .. }, Ty::Str(_)) => {
                let info = &self.items.structs[id.0 as usize];
                let name = match &operand.kind {
                    ast::ExprKind::Name(n) => format!("'{}'", n.name),
                    _ => "…".to_string(),
                };
                let field = info.fields.first().map(|f| format!(".{}", f.name)).unwrap_or_default();
                return err(Diagnostic::error(format!("a `{}` is not text: build the text you want from its fields, e.g. `to.str.utf8({}{}) ++ …`, or print it directly (owner, 2026-09-30)", info.name, name, field)).named("not_text").at(span));
            }
            (Ty::Map { .. }, Ty::Str(_)) => return err(Diagnostic::error("a map is not text: build the text you want from its entries, or print it directly (owner, 2026-09-30)").named("not_text").at(span)),
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
            (Ty::Char(_), Ty::Int(_)) => return err(Diagnostic::error("a character does not convert to a number: `to.i32(`7`)` could mean 55 (its code) or 7 (its digit) — `char.code(…)` gives the code, `to.i32(to.str.utf8(…)) otherwise …` the digit").named("no_such_conversion").at(span)),
            (Ty::Int(_), Ty::Char(_)) => return err(Diagnostic::error("a number does not convert to a character: `char.point.from_code(…) otherwise …` builds one from a code").named("no_such_conversion").at(span)),
            (_, Ty::Bool) => return err(Diagnostic::error("nothing converts to `bool`: there is no truthiness — compare instead (`$n not_equal 0`)").named("no_truthiness").at(span)),
            // Bytes read as text (owner, 2026-09-30: "yea"): UTF-8 only.
            (Ty::Seq { elem, .. }, Ty::Str(enc)) if matches!(elem.base(), Ty::Int(IntW::Fixed { signed: false, bits: 8 })) => {
                if *enc != Enc::Utf8 {
                    return err(Diagnostic::error(format!("only `to.str.utf8` reads bytes as text (owner, 2026-09-30); `to.{}` from bytes is not decided — convert to utf8 first", target_spelling)).named("no_such_conversion").at(span));
                }
                fails.push("not_utf8");
            }
            (Ty::Seq { .. }, Ty::Str(_)) => return err(Diagnostic::error("a list is not text: only a list of `u8` converts, read as UTF-8 bytes (`to.str.utf8($bytes) otherwise …`); join the items with `concat`, or print the list directly (owner, 2026-09-30)").named("not_text").at(span)),
            (_, Ty::Seq { .. }) => return err(Diagnostic::error("there is no conversion to a list").named("no_such_conversion").at(span)),
            // A number from C becomes a C enum only through a check (owner,
            // 2026-09-27): one no case has fails `out_of_range`.
            (Ty::Int(_), Ty::Enum { id, .. }) if self.items.enums[id.0 as usize].numbers.is_some() => fails.push("out_of_range"),
            (Ty::Int(_), Ty::Enum { .. }) => return err(Diagnostic::error(format!("only a `layout.C enum` converts from a number, and `{}` is a Cyborg enum: its value is built by name", target_spelling)).named("cnc::numbered_enum_not_c").at(span)),
            // And back: a C enum's number never fails (owner, 2026-09-28);
            // a narrower target is a narrowing site, as ever.
            (Ty::Enum { id, .. }, Ty::Int(_)) if self.items.enums[id.0 as usize].numbers.is_some() => {}
            (Ty::Enum { .. }, Ty::Int(_)) => return err(Diagnostic::error(format!("only a `layout.C enum` has numbers: `{}` is a Cyborg enum, whose cases are names", self.spell(&vt))).named("cnc::numbered_enum_not_c").at(span)),
            (_, Ty::Struct { .. }) | (_, Ty::Enum { .. }) => return err(Diagnostic::error(format!("there is no conversion to `{}`: a struct or enum value is built by name", target_spelling)).named("no_such_conversion").at(span)),
            (Ty::Duration, _) | (_, Ty::Duration) => return err(Diagnostic::error("a duration does not convert: compare or add durations directly").named("no_such_conversion").at(span)),
            _ => return err(Diagnostic::error(format!("there is no conversion from `{}` to `{}`", self.spell(&vt), target_spelling)).named("no_such_conversion").at(span)),
        }
        // Into a range type: fails out_of_range (never a check site).
        if t.bound().is_some() && !fails.contains(&"out_of_range") {
            fails.push("out_of_range");
        }
        if useful && !fails.is_empty() && matches!(t.base(), Ty::Str(_)) && matches!(vt.base(), Ty::Bin(_) | Ty::Deci(_) | Ty::Enum { .. }) {
            return err(Diagnostic::error(format!("not yet: `to.useful.{}` of {} into text that can fail has no shape in the generated code yet", target_spelling, crate::types::a_type(&self.spell(&vt)))).named("not_built_yet").at(span));
        }
        let fail_list: Vec<(String, Option<Ty>)> = fails.iter().map(|f| (f.to_string(), None)).collect();
        let ow = match otherwise {
            Some(o) => {
                if fails.is_empty() {
                    return err(Diagnostic::error(format!("`to.{}` cannot fail here; drop the `otherwise`", target_spelling)).named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                Some(Box::new(self.otherwise(ctx, o, &fail_list, &t, used, span)?))
            }
            None => {
                if !fails.is_empty() && !ctx.must_fail {
                    return err(Diagnostic::error(format!("`to.{}` can fail ({}): say what happens — `otherwise …`", target_spelling, fails.join(", "))).named("failure_unhandled").at(span));
                }
                None
            }
        };
        // Asked last, as the oracle does: what cannot convert at all, or
        // cannot fail, is said first.
        if form.is_none() {
            let two_texts = matches!(t.base(), Ty::Str(_)) && matches!(vt.base(), Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) | Ty::Bool | Ty::Enum { .. } | Ty::Brain);
            if two_texts {
                return err(Diagnostic::error(format!("say which text `to.{}` makes of {}: `to.stored.{}(…)` (what is stored) or `to.useful.{}(…)` (the useful value) (owner, 2026-10-02)", target_spelling, self.family_word(&vt), target_spelling, target_spelling)).named("conversion_target_unsaid").at(span).fix_edit(span, Edit::Swap(vec![format!("to.{}", target_spelling)]), format!("to.useful.{}", target_spelling)).fix_edit(span, Edit::Swap(vec![format!("to.{}", target_spelling)]), format!("to.stored.{}", target_spelling)));
            }
            if matches!(t.base(), Ty::Deci(_)) && vt.is_brain() {
                return err(Diagnostic::error(format!("say which value `to.{}` takes of a brain16: `to.stored.{}(…)` (what is stored) or `to.useful.{}(…)` (the useful value) (owner, 2026-10-02)", target_spelling, target_spelling, target_spelling)).named("ai::brain_to_deci_unsaid").at(span).fix_edit(span, Edit::Swap(vec![format!("to.{}", target_spelling)]), format!("to.useful.{}", target_spelling)).fix_edit(span, Edit::Swap(vec![format!("to.{}", target_spelling)]), format!("to.stored.{}", target_spelling)));
            }
            if matches!(t.base(), Ty::Deci(_)) && matches!(vt.base(), Ty::Bin(_)) {
                return err(Diagnostic::error(format!("say which value `to.{}` takes of a bin: `to.stored.{}(…)` (what is stored) or `to.useful.{}(…)` (the useful value) (owner, 2026-10-02)", target_spelling, target_spelling, target_spelling)).named("money::bin_to_deci_unsaid").at(span).fix_edit(span, Edit::Swap(vec![format!("to.{}", target_spelling)]), format!("to.useful.{}", target_spelling)).fix_edit(span, Edit::Swap(vec![format!("to.{}", target_spelling)]), format!("to.stored.{}", target_spelling)));
            }
        }
        if written && !(matches!(t.base(), Ty::Deci(_)) && matches!(vt.base(), Ty::Str(_))) {
            return err(Diagnostic::error(format!("`.written` counts the digits of text as written into a deci, trailing zeros included — `to.d32.written(\"1234567.0\")`; `to.{}.written` of {} has nothing written to count", target_spelling, crate::types::a_type(&self.spell(&vt)))).named("conversion_form").at(span));
        }
        let fails_owned: Vec<String> = fails.iter().map(|s| s.to_string()).collect();
        Ok(Expr { kind: K::Conv { target: t.clone(), expr: Box::new(v), otherwise: ow, fails: fails_owned, useful, bits: false, written }, ty: t, span })
    }

    /// `to.i8.bits[…]` (owner, 2026-10-06): a whole number's bits read as
    /// the type's — `to.i8.bits[128]` is -128, `to.u8.bits[-1]` is 255.
    /// A value from -2^(N-1) to 2^N - 1 has them (design claude's reading,
    /// not ruled); what `.bits` means past whole numbers is not decided.
    fn to_bits(&mut self, ctx: &mut FnCtx, target: &ast::Type, operand: &ast::Expr, otherwise: Option<&ast::Otherwise>, span: Span) -> Result<Expr> {
        let mut blanks = Vec::new();
        let t = self.items.resolve_type(target, ctx.module, &mut blanks, false)?;
        let n = match &t {
            Ty::Int(IntW::Fixed { bits, .. }) if t.exact_limit().is_none() => *bits,
            _ => return err(Diagnostic::error(format!("`.bits` is for whole-number types: what `to.{}.bits` means is not decided", self.spell(&t))).named("needs_integer").at(span)),
        };
        if otherwise.is_some() {
            return err(Diagnostic::error(format!("`to.{}.bits` cannot fail: a value past its bits is a check site, not a failure; drop the `otherwise`", self.spell(&t))).named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
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
                    return err(Diagnostic::error(format!("`{}` does not fit the {} bits of {}: `to.{}.bits` reads a pattern of {} bits", i, n, self.spell(&t), self.spell(&t), n)).named("value_does_not_fit").at(v.span));
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
            return err(Diagnostic::error(format!("`to.{}.bits` reads a whole number's bits, and this is {}", self.spell(&t), article(&self.spell(&vt)))).named("needs_integer").at(v.span));
        }
        Ok(Expr { kind: K::Conv { target: t.clone(), expr: Box::new(v), otherwise: None, fails: Vec::new(), useful: false, bits: true, written: false }, ty: t, span })
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
            ("list", ["empty"]) => return err(Diagnostic::error("`list.empty` is a value, not a call: write it without brackets").named("not_a_function").at(span)),
            ("list", [other]) => return err(Diagnostic::error(format!("`list.{}` does not exist: a list has `length`, `append`, `remove`, `slice`", other)).named("unknown_operation").at(span).fix_swap_close(span, "list", other, ["length", "append", "remove", "slice"])),
            ("array", ["length"]) => BuiltinHead::B(Builtin::ArrayLength),
            ("array", ["slice"]) => BuiltinHead::B(Builtin::ArraySlice),
            ("array", [other]) => return err(Diagnostic::error(format!("`array.{}` does not exist: an array has `length` and `slice`", other)).named("unknown_operation").at(span).fix_swap_close(span, "array", other, ["length", "slice"])),
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
            ("map", [other]) => return err(Diagnostic::error(format!("`map.{}` does not exist: a map has `length`, `has`, `get`, `set`, `remove`, `keys`, `values`, `pairs`", other)).named("unknown_operation").at(span).fix_swap_close(span, "map", other, ["length", "has", "get", "set", "remove", "keys", "values", "pairs"])),
            ("str", ["bytes"]) => BuiltinHead::B(Builtin::StrBytes),
            ("str", ["points"]) => BuiltinHead::B(Builtin::StrPoints),
            ("str", ["graphemes"]) => BuiltinHead::B(Builtin::StrGraphemes),
            ("str", ["length"]) => return err(Diagnostic::error("a text has no single length: `str.bytes`, `str.points` or `str.graphemes` — say which").named("text_unit_unsaid").at(span).fix_edit(span, Edit::Swap(vec!["str.length".into()]), "str.bytes").fix_edit(span, Edit::Swap(vec!["str.length".into()]), "str.points").fix_edit(span, Edit::Swap(vec!["str.length".into()]), "str.graphemes")),
            ("str", ["each"]) => return err(Diagnostic::error("say the unit after `str.each`: `str.each.grapheme(…)`, `.point`, or `.byte`").named("text_unit_unsaid").at(span).fix_edit(span, Edit::Swap(vec!["str.each".into()]), "str.each.grapheme").fix_edit(span, Edit::Swap(vec!["str.each".into()]), "str.each.point").fix_edit(span, Edit::Swap(vec!["str.each".into()]), "str.each.byte")),
            ("str", ["each", unit]) => match *unit {
                "byte" => BuiltinHead::B(Builtin::StrEach(Unit::Byte)),
                "point" => BuiltinHead::B(Builtin::StrEach(Unit::Point)),
                "grapheme" => BuiltinHead::B(Builtin::StrEach(Unit::Grapheme)),
                other => return err(Diagnostic::error(format!("`str.each.{}` is not a unit: `byte`, `point` or `grapheme`", other)).named("unknown_text_unit").at(span).fix_swap_close(span, "each", other, ["byte", "point", "grapheme"])),
            },
            ("str", [other, ..]) => return err(Diagnostic::error(format!("`str.{}` is std's, not the language's: the language owns only the units (`str.bytes`, `str.each.grapheme`, …), and `std::` is not available yet", other)).named(if matches!(*other, "upper" | "lower" | "title" | "fold") { "case_mapping_unsaid" } else { "std_name_needs_path" }).at(span)),
            ("char", ["code"]) => BuiltinHead::B(Builtin::CharCode),
            // Built by the parser from `on.machine.chip X` (owner, 2026-10-06).
            ("on", ["machine", "chip", f]) => match ast::ON_FEATURES.iter().position(|n| n == f) {
                Some(i) => BuiltinHead::B(Builtin::MachineHas(i as u32)),
                None => return Ok(None),
            },
            ("char", [unit, "from_code"]) => match *unit {
                "byte" => BuiltinHead::B(Builtin::CharFromCode(Unit::Byte)),
                "point" => BuiltinHead::B(Builtin::CharFromCode(Unit::Point)),
                "grapheme" => return err(Diagnostic::error("a grapheme has no single code: `char.point.from_code(…)` or `char.byte.from_code(…)`").named("unicode::no_single_code").at(span)),
                other => return err(Diagnostic::error(format!("`char.{}` is not a unit", other)).named("unknown_text_unit").at(span)),
            },
            ("char", [..]) => return err(Diagnostic::error(format!("`{}` does not exist: `char.code(…)`, `char.point.from_code(…)`, `char.byte.from_code(…)`", path.spelled())).named("unknown_operation").at(span)),
            ("ptr", ["read"]) => BuiltinHead::B(Builtin::PtrRead),
            // C strings (owner, 2026-10-19), C's own vocabulary under `C.`.
            ("C", ["str", "read"]) => BuiltinHead::B(Builtin::CStrRead),
            ("C", ["str", "make"]) => BuiltinHead::B(Builtin::CStrMake),
            ("C", ["str", ..]) => return err(Diagnostic::error("C strings are `C.str.read($p)` and `C.str.make(lend $s)`: a C string read into a text, and a text made into one").named("cnc::c_str_unknown").at(span)),
            ("ptr", ["write"]) => BuiltinHead::B(Builtin::PtrWrite),
            ("ptr", ["at"]) => BuiltinHead::B(Builtin::PtrAt),
            ("ptr", ["is_null"]) => BuiltinHead::B(Builtin::PtrIsNull),
            ("ptr", ["null"]) => return err(Diagnostic::error("`ptr.null` is the null pointer, a value, not a call: `free(ptr.null)`").named("not_a_function").at(span)),
            ("ptr", [other]) => return err(Diagnostic::error(format!("`ptr.{}` does not exist: a pointer has `read`, `write`, `at`, `is_null`, and `ptr.null` is the null pointer", other)).named("unknown_pointer_operation").at(span).fix_swap_close(span, "ptr", other, ["read", "write", "at", "is_null"])),
            ("simd", ["fill"]) => BuiltinHead::B(Builtin::SimdFill),
            ("simd", ["load"]) => BuiltinHead::B(Builtin::SimdLoad),
            ("simd", ["store"]) => BuiltinHead::B(Builtin::SimdStore),
            ("simd", ["all"]) => BuiltinHead::B(Builtin::SimdAll),
            ("simd", ["any"]) => BuiltinHead::B(Builtin::SimdAny),
            ("simd", ["max"]) => BuiltinHead::B(Builtin::SimdMax),
            ("simd", ["min"]) => BuiltinHead::B(Builtin::SimdMin),
            ("simd", ["sum"]) => BuiltinHead::B(Builtin::SimdSum),
            ("simd", ["sum", "lane"]) => BuiltinHead::B(Builtin::SimdSumLane),
            ("simd", ["sum", "wrap"]) => BuiltinHead::B(Builtin::SimdSumWrap),
            ("simd", ["shuffle"]) => BuiltinHead::B(Builtin::SimdShuffle(Vec::new())),
            ("simd", ["load", "some"]) => BuiltinHead::B(Builtin::SimdLoadSome),
            ("simd", ["store", "some"]) => BuiltinHead::B(Builtin::SimdStoreSome),
            ("simd", [other]) => return err(Diagnostic::error(format!("`simd.{}` does not exist: a vector has `fill`, `load`, `store`, `load.some`, `store.some`, `shuffle`, `all`, `any`, `max`, `min`, `sum`", other)).named("unknown_operation").at(span)),
            ("channel", ["send"]) => BuiltinHead::B(Builtin::ChannelSend),
            ("channel", ["take"]) => BuiltinHead::B(Builtin::ChannelTake),
            ("channel", [other]) => return err(Diagnostic::error(format!("`channel.{}` does not exist: a channel has `send` and `take`", other)).named("unknown_operation").at(span).fix_swap_close(span, "channel", other, ["send", "take"])),
            _ => return Ok(None),
        };
        Ok(Some(b))
    }

    fn builtin_call(&mut self, ctx: &mut FnCtx, head: BuiltinHead, path: &ast::Path, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let b = match head {
            BuiltinHead::MapLiteral => {
                if otherwise.is_some() {
                    return err(Diagnostic::error("a map literal cannot fail; drop the `otherwise`").named("otherwise_never_needed").at(span).fix_edit(span, Edit::CutFrom("otherwise".into()), ""));
                }
                return self.map_literal(ctx, args, span);
            }
            BuiltinHead::B(b) => b,
        };
        let name = path.spelled();
        let labelled;
        let args = if matches!(b, Builtin::ListSlice | Builtin::ArraySlice) {
            labelled = slice_args(&name, args, span)?;
            &labelled[..]
        } else {
            args
        };
        let positional: Vec<&ast::Expr> = {
            let mut v = Vec::new();
            for a in args {
                match a {
                    AArg::Positional(e) => v.push(e),
                    _ => return err(Diagnostic::error(format!("`{}` takes values in order", name)).named("argument_label_unknown").at(a.span())),
                }
            }
            v
        };
        let arity = |n: usize, this: &Self| -> Result<()> {
            if positional.len() != n {
                return err(Diagnostic::error(format!("`{}` takes {} argument{}, and {} {} given", name, n, if n == 1 { "" } else { "s" }, positional.len(), if positional.len() == 1 { "was" } else { "were" })).named("wrong_argument_count").at(span));
            }
            let _ = this;
            Ok(())
        };
        let mut fails: Vec<(String, Option<Ty>)> = Vec::new();
        let mut shuffle_order: Option<Vec<u32>> = None;
        let (hargs, ty): (Vec<Arg>, Ty) = match &b {
            Builtin::MustReport | Builtin::MustStopFork | Builtin::MustStopDone | Builtin::InclBytes | Builtin::ConstView | Builtin::PtrNull => unreachable!("made by `must`, `incl`, a const or `ptr.null`, never called"),
            Builtin::ListLength | Builtin::ArrayLength | Builtin::MapLength | Builtin::StrBytes | Builtin::StrPoints | Builtin::StrGraphemes => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                match (&b, at.base()) {
                    (Builtin::ListLength, Ty::Seq { kind: SeqKind::Array, .. }) => return err(Diagnostic::error(format!("{} is an `array`: its length is `array.length(…)`", self.arg_name(ctx, &a))).named("list_array_mismatch").at(span)),
                    (Builtin::ListLength, Ty::Seq { .. } | Ty::Bundle(_)) => {}
                    (Builtin::ArrayLength, Ty::Seq { kind: SeqKind::List, .. }) => return err(Diagnostic::error(format!("{} is a `list`: its length is `list.length(…)`", self.arg_name(ctx, &a))).named("list_array_mismatch").at(span)),
                    (Builtin::ArrayLength, Ty::Seq { .. }) => {}
                    (Builtin::MapLength, Ty::Map { .. }) => {}
                    (Builtin::StrBytes | Builtin::StrPoints | Builtin::StrGraphemes, Ty::Str(_)) => {}
                    (Builtin::ListLength | Builtin::ArrayLength, Ty::Str(_)) => return err(Diagnostic::error("a text has no single length: `str.bytes`, `str.points` or `str.graphemes` — say which").named("text_unit_unsaid").at(span).fix_edit(span, Edit::Swap(vec!["std::list.length".into(), "list.length".into(), "array.length".into()]), "str.bytes").fix_edit(span, Edit::Swap(vec!["std::list.length".into(), "list.length".into(), "array.length".into()]), "str.points").fix_edit(span, Edit::Swap(vec!["std::list.length".into(), "list.length".into(), "array.length".into()]), "str.graphemes")),
                    _ => return err(Diagnostic::error(format!("`{}` takes a {}, and {} is {}", name, builtin_subject(&b), self.arg_name(ctx, &a), crate::types::a_type(&self.spell(&at)))).named("wrong_type").at(positional[0].span)),
                }
                let mut ty = Ty::u64();
                // A bounded list's length is bounded too.
                if let Some(Bound::Len { lo, hi, .. }) = at.bound() {
                    ty = Ty::Bounded { inner: Box::new(Ty::u64()), bound: Bound::Int { lo: *lo as i128, hi: *hi as i128, said: Default::default() } };
                }
                (vec![a], ty)
            }
            // A list of `count` copies of one value (owner, 2026-09-30). A
            // strict file labels the count; an owned value is copied, so a
            // bare name of one says `copy`.
            Builtin::ListFill => {
                arity(2, self)?;
                let ce = match &positional[1].kind {
                    AK::Count(inner) => &**inner,
                    _ => return err(Diagnostic::error("the count says `count`: `list.fill(value: 0, count: 10)`").named("argument_label_missing").at(positional[1].span).fix_edit(positional[1].span, Edit::Label, "count: ")),
                };
                let value_e = match &positional[0].kind {
                    AK::Value(inner) => &**inner,
                    _ => return err(Diagnostic::error("the value says `value`: `list.fill(value: 0, count: 10)`").named("argument_label_missing").at(positional[0].span).fix_edit(positional[0].span, Edit::Label, "value: ")),
                };
                if let AK::Name(n) = &value_e.kind {
                    if self.peek_ty(ctx, value_e).is_some_and(|t| self.resolve(&t).is_owned()) {
                        return err(Diagnostic::error(format!("`list.fill` makes copies of '{}': `copy ${}`", n.name, n.name)).named("owned_value_bare").at(value_e.span).fix_edit(value_e.span, Edit::Verb, "copy"));
                    }
                }
                let v = match &value_e.kind {
                    AK::Verb { expr, .. } => {
                        let Some(pt) = self.peek_ty(ctx, expr) else {
                            return err(Diagnostic::error("`list.fill` fills with a value, or a `copy` or `give` of one").named("verb_wrong_for_parameter").at(value_e.span));
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
                    let to = if plain_group(&c) { exact_for(&c) } else { Ty::u64() };
                    retype_literal_arith(&mut c, &to);
                }
                if !self.resolve(&c.ty).is_int() {
                    return err(Diagnostic::error(format!("`list.fill`'s count is a whole number, and this is {}", crate::types::a_type(&self.spell(&self.resolve(&c.ty))))).named("wrong_type").at(ce.span));
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
                            return err(Diagnostic::error(format!("{} is an `array`, which cannot grow or shrink: declare it a `list` to append", self.arg_name(ctx, &a))).named("array_cannot_grow").at(span));
                        }
                        (**elem).clone()
                    }
                    other => return err(Diagnostic::error(format!("`list.append` takes a list, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                let v = self.one_argument(ctx, positional[1], ParamKind::Owned, &elem, "list.append", "the element")?;
                if let Arg::Lend { .. } = v {
                    return err(Diagnostic::error("a list owns its elements: `append` takes a fresh value, a `copy`, or a `give` — never a lend").named("verb_wrong_for_parameter").at(positional[1].span).fix_edit(positional[1].span, Edit::Verb, "copy").fix_edit(positional[1].span, Edit::Verb, "give"));
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
                            return err(Diagnostic::error(format!("{} is an `array`, which cannot grow or shrink", self.arg_name(ctx, &a))).named("array_cannot_grow").at(span));
                        }
                        (**elem).clone()
                    }
                    other => return err(Diagnostic::error(format!("`list.remove` takes a list, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                // `pos` on every position (owner, 2026-10-07: "`a`").
                let AK::Pos(at) = &positional[1].kind else {
                    return err(Diagnostic::error("the position says `pos`: `list.remove(lend.mut $xs, pos: 2)`").named("argument_label_missing").at(positional[1].span).fix_edit(positional[1].span, Edit::Label, "pos: "));
                };
                let i = self.expr(ctx, at)?;
                if !self.resolve(&i.ty).is_int() {
                    return err(Diagnostic::error("`list.remove` takes a 1-based position").named("wrong_type").at(positional[1].span));
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
                    (Builtin::ListSlice, Ty::Seq { kind: SeqKind::Array, .. }) => return err(Diagnostic::error(format!("{} is an `array`: its slice is `array.slice(…)`", self.arg_name(ctx, &a))).named("list_array_mismatch").at(span)),
                    (_, Ty::Seq { kind: SeqKind::List, .. }) => return err(Diagnostic::error(format!("{} is a `list`: its slice is `list.slice(…)`", self.arg_name(ctx, &a))).named("list_array_mismatch").at(span)),
                    (_, other) => return err(Diagnostic::error(format!("`{}` takes a {}, and {} is {}", name, builtin_subject(&b), self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
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
                    AK::Name(n) => return err(Diagnostic::error(format!("converting bare '{}' would copy it without saying so: `give ${}` hands it over, `copy ${}` keeps it", n.name, n.name, n.name)).named("owned_value_bare").at(e.span)),
                    AK::Verb { verb: Verb::Lend | Verb::LendMut, .. } => return err(Diagnostic::error("converting a lend would copy it without saying so: `copy` it or `give` it").named("verb_wrong_for_parameter").at(e.span).fix_edit(positional[0].span, Edit::Verb, "copy").fix_edit(positional[0].span, Edit::Verb, "give")),
                    AK::Path(p) if p.kit.is_none() && p.segs.len() == 1 && (crate::resolve::primitive(p.head()).is_some() || matches!(p.head(), "list" | "array" | "map")) => {
                        return err(Diagnostic::error(format!("`{}` keeps the element type, so it is not written: `{}(copy $a)`", name, name)).named("conversion_form").at(e.span));
                    }
                    _ => {}
                }
                let elem = self.fresh();
                let any = Ty::Seq { elem: Box::new(elem), kind: SeqKind::Free(InferId(0)) };
                let a = self.one_argument(ctx, e, ParamKind::Owned, &any, &name, "the list or array")?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Seq { elem, .. } => (**elem).clone(),
                    other => return err(Diagnostic::error(format!("`to.list` / `to.array` take a list or an array, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(e.span)),
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
                    other => return err(Diagnostic::error(format!("`{}` takes a map, and {} is {}", name, self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
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
                    other => return err(Diagnostic::error(format!("`map.set` takes a map, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                let k = self.one_argument(ctx, positional[1], ParamKind::Owned, &kt, "map.set", "the key")?;
                let v = self.one_argument(ctx, positional[2], ParamKind::Owned, &vt, "map.set", "the value")?;
                if matches!(k, Arg::Lend { .. }) || matches!(v, Arg::Lend { .. }) {
                    return err(Diagnostic::error("a map owns its keys and values: `set` takes a fresh value, a `copy`, or a `give` — never a lend").named("verb_wrong_for_parameter").at(span));
                }
                (vec![a, k, v], Ty::Nothing)
            }
            Builtin::MapKeys | Builtin::MapValues | Builtin::MapPairs => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let (kt, vt, order) = match at.base() {
                    Ty::Map { key, val, order } => ((**key).clone(), (**val).clone(), *order),
                    other => return err(Diagnostic::error(format!("`{}` takes a map, and {} is {}", name, self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                if order == MapOrder::Unordered {
                    return err(Diagnostic::error(format!("{} is a `map.unordered`, which promises never to be walked: `{}` needs a `map` or `map.ordered`", self.arg_name(ctx, &a), name)).named("unordered_map_walked").at(span));
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
                    return err(Diagnostic::error(format!("`{}` walks a text, and {} is {}", name, self.arg_name(ctx, &a), crate::types::a_type(&self.spell(&at)))).named("wrong_type").at(positional[0].span));
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
                    Ty::Char(Unit::Grapheme) => return err(Diagnostic::error("a grapheme has no single code: it may be several code points — walk it with `str.each.point` after `to.str.utf8(…)`").named("unicode::no_single_code").at(span)),
                    Ty::Char(_) => Ty::int(false, 32),
                    other => return err(Diagnostic::error(format!("`char.code` takes a character, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                (vec![Arg::Value(v)], ty)
            }
            Builtin::CharFromCode(unit) => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                let vt = self.resolve(&v.ty);
                if !vt.is_int() {
                    return err(Diagnostic::error(format!("`{}` takes a number (the code), and this is {}", name, crate::types::a_type(&self.spell(&vt)))).named("wrong_type").at(positional[0].span));
                }
                fails.push(("not_a_char".into(), None));
                (vec![Arg::Value(v)], Ty::Char(*unit))
            }
            // A C string read up to its 0 byte into a text (owner,
            // 2026-10-19): from a pointer C handed over, or a C string this
            // program made (`lend 'cs'`). Fails `not_utf8`.
            Builtin::CStrRead => {
                arity(1, self)?;
                let a = match &positional[0].kind {
                    AK::Verb { .. } => self.lent_arg(ctx, positional[0], &name, false)?,
                    _ => Arg::Value(self.expr(ctx, positional[0])?),
                };
                let at = self.resolve(&self.arg_type(&a));
                let bytes = |t: &Ty| matches!(t.base(), Ty::Int(IntW::Fixed { bits: 8, .. }));
                match at.base() {
                    Ty::Ptr { elem, .. } if bytes(elem) => {}
                    Ty::Seq { elem, .. } if bytes(elem) => {}
                    other => return err(Diagnostic::error(format!("`C.str.read` reads bytes ending in a 0: a `ptr[u8]` or a C string `C.str.make` made, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                }
                fails.push(("not_utf8".into(), None));
                (vec![a], Ty::utf8())
            }
            // A text's bytes and a 0, what C reads (owner, 2026-10-19). A
            // literal with no 0 byte cannot fail, so it needs no
            // `otherwise`; anything else may hold one, and fails
            // `has_zero_byte`.
            Builtin::CStrMake => {
                arity(1, self)?;
                let a = match &positional[0].kind {
                    AK::Verb { .. } => self.lent_arg(ctx, positional[0], &name, false)?,
                    _ => Arg::Value(self.expr(ctx, positional[0])?),
                };
                let at = self.resolve(&self.arg_type(&a));
                if !matches!(at.base(), Ty::Str(_)) {
                    return err(Diagnostic::error(format!("`C.str.make` makes a C string of a text, and this is {}", crate::types::a_type(&self.spell(&at)))).named("wrong_type").at(positional[0].span));
                }
                let literal = match &a {
                    Arg::Value(Expr { kind: K::Str(s), .. }) => Some(s.clone()),
                    _ => None,
                };
                let zero_fail = !literal.as_ref().is_some_and(|s| !s.contains('\0'));
                if zero_fail {
                    fails.push(("has_zero_byte".into(), None));
                }
                // Written straight into a bounded `C.str`, it is made that
                // type, and fails `out_of_range` when it does not fit (design
                // claude's reading, as any conversion into a range type);
                // a literal that fits cannot.
                let cstr = Ty::Seq { elem: Box::new(Ty::int(false, 8)), kind: SeqKind::CStr(true) };
                let hint = self.store_hint.clone().filter(|(t, s)| *s == span && matches!(t.base(), Ty::Seq { kind: SeqKind::CStr(_), .. }));
                let mut args = vec![a];
                let ty = match hint {
                    Some((t, _)) => {
                        if let Some(Bound::Len { lo, hi, .. }) = t.bound() {
                            let fits = literal.as_ref().is_some_and(|s| (s.len() as u64 + 1) >= *lo && (s.len() as u64 + 1) <= *hi);
                            if !fits {
                                fails.push(("out_of_range".into(), None));
                                args.push(Arg::Value(Expr { kind: K::Bool(zero_fail), ty: Ty::Bool, span }));
                                args.push(Arg::Value(Expr { kind: K::Int(*lo as i128), ty: Ty::u64(), span }));
                                args.push(Arg::Value(Expr { kind: K::Int(*hi as i128), ty: Ty::u64(), span }));
                            }
                        }
                        t
                    }
                    None => cstr,
                };
                (args, ty)
            }
            Builtin::PtrRead | Builtin::PtrWrite | Builtin::PtrAt | Builtin::PtrIsNull => {
                match b {
                    Builtin::PtrRead => self.power_use(ctx, &["raw_memory", "read"], None, "reads raw memory (`ptr.read`)".into(), span),
                    Builtin::PtrAt => self.power_use(ctx, &["raw_memory", "read"], None, "reads raw memory (`ptr.at`)".into(), span),
                    Builtin::PtrWrite => self.power_use(ctx, &["raw_memory", "write"], None, "writes raw memory (`ptr.write`)".into(), span),
                    _ => {}
                }
                if b != Builtin::PtrIsNull && self.needs_mark(ctx) {
                    self.power_first()?;
                    return err(Diagnostic::error(format!("`{}` is a no_check site: nothing about C's memory can be proven — write `no_check` in front of the statement", name)).named("no_check_missing").at(span));
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
                    other => return err(Diagnostic::error(format!("`{}` takes a pointer, and this is {}", name, crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
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
                                return err(Diagnostic::error(format!("`ptr.read` of a `{}` from C can fail (out_of_range): C may hold a number `{}` does not allow — say what happens with `otherwise`", self.spell(&elem), self.spell(&elem))).named("failure_unhandled").at(span));
                            }
                            fails.push(("out_of_range".into(), None));
                        }
                        (vec![Arg::Value(p)], elem)
                    }
                    Builtin::PtrIsNull => (vec![Arg::Value(p)], Ty::Bool),
                    Builtin::PtrAt => {
                        let i = self.expr(ctx, positional[1])?;
                        if !self.resolve(&i.ty).is_int() {
                            return err(Diagnostic::error("`ptr.at` moves by a whole number of elements").named("wrong_type").at(positional[1].span));
                        }
                        (vec![Arg::Value(p), Arg::Value(i)], pt.base().clone())
                    }
                    _ => {
                        if !mutable {
                            return err(Diagnostic::error("`ptr.write` needs a `ptr.mut`: this pointer is read-only").named("pointer_read_only").at(span));
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
                    other => return err(Diagnostic::error(format!("`channel.send` takes a channel, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                let v = self.one_argument(ctx, positional[1], ParamKind::Owned, &elem, "channel.send", "the value")?;
                if let Arg::Lend { .. } = v {
                    return err(Diagnostic::error("a value sent on a channel is given, never lent: no two tasks may hold one thing").named("mcps::send_needs_give").at(positional[1].span).fix_edit(positional[1].span, Edit::Verb, "give"));
                }
                (vec![a, v], Ty::Nothing)
            }
            Builtin::ChannelTake => {
                arity(1, self)?;
                let a = self.lent_arg(ctx, positional[0], &name, false)?;
                let at = self.resolve(&self.arg_type(&a));
                let elem = match at.base() {
                    Ty::Channel(e) => (**e).clone(),
                    other => return err(Diagnostic::error(format!("`channel.take` takes a channel, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                fails.push(("closed".into(), None));
                (vec![a], elem)
            }
            Builtin::SimdFill => {
                arity(1, self)?;
                // `simd.fill[value 0]` (owner, 2026-10-07: "`a`").
                let AK::Value(fill) = &positional[0].kind else {
                    return err(Diagnostic::error("the value says `value`: `simd.fill(value: 0)`").named("argument_label_missing").at(positional[0].span).fix_edit(positional[0].span, Edit::Label, "value: "));
                };
                let mut v = self.expr(ctx, fill)?;
                // Bare numbers alone are worked out exactly (dc83c05): left
                // with an open width, `15059671820752541943 div 3` was done
                // at a signed 64 bits, where the literal is negative, in any
                // build that did not fold it (check claude, seed 5120).
                if literal_only(&v) && !is_int_literal(&v) && matches!(self.resolve(&v.ty).base(), Ty::Int(IntW::Free(_))) {
                    let to = exact_for(&v);
                    retype_literal_arith(&mut v, &to);
                }
                // An answer of plain math is exact, and lands where it is
                // stored (owner, 2026-09-23): in every lane, whose type the
                // vector takes from its place. As the lane type it made a
                // vector of 65-bit lanes, one to a register — `simd.fill['x'
                // + 1]` into a `simd[i32, 4]` was a `simd[integer, 1]`
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
                    other => return err(Diagnostic::error(format!("`simd.load` fills a vector from a list or an array, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
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
                    other => return err(Diagnostic::error(format!("`simd.store` writes into a list or an array, and {} is {}", self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                let idx = self.pos_arg(ctx, positional[1], &name)?;
                let v = self.expr(ctx, positional[2])?;
                let vt = self.resolve(&v.ty);
                match vt.base() {
                    Ty::Simd { elem: lane, .. } => {
                        if !self.unify(lane, &elem) {
                            return err(Diagnostic::error(format!("`simd.store` writes `{}` lanes into a list of `{}`", self.spell(lane), self.spell(&elem))).named("wrong_type").at(positional[2].span));
                        }
                    }
                    other => return err(Diagnostic::error(format!("`simd.store` writes a vector, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[2].span)),
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
                    other => return err(Diagnostic::error(format!("`{}` works on a list or an array, and {} is {}", name, self.arg_name(ctx, &a), crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                let idx = self.pos_arg(ctx, positional[1], &name)?;
                let k = self.expr(ctx, positional[2])?;
                if !self.resolve(&k.ty).is_int() {
                    return err(Diagnostic::error(format!("`{}`'s third argument is how many items, a whole number from 0 to the lanes, and this is {}", name, crate::types::a_type(&self.spell(&self.resolve(&k.ty))))).named("wrong_type").at(positional[2].span));
                }
                if !store {
                    (vec![a, idx, Arg::Value(k)], Ty::Simd { elem: Box::new(elem), lanes: 0 })
                } else {
                    let v = self.expr(ctx, positional[3])?;
                    let vt = self.resolve(&v.ty);
                    match vt.base() {
                        Ty::Simd { elem: lane, .. } => {
                            if !self.unify(lane, &elem) {
                                return err(Diagnostic::error(format!("`simd.store.some` writes `{}` lanes into a list of `{}`", self.spell(lane), self.spell(&elem))).named("wrong_type").at(positional[3].span));
                            }
                        }
                        other => return err(Diagnostic::error(format!("`simd.store.some` writes a vector, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[3].span)),
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
                    Ty::Simd { .. } => return err(Diagnostic::error("`simd.shuffle` needs to know how many lanes the vector has: give it a type (`let $v: simd[i32, count: 4] = …`)").named("simd_lane_count_unknown").at(positional[0].span)),
                    other => return err(Diagnostic::error(format!("`simd.shuffle` rearranges a vector's lanes, and this is {}", crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                let fixed = "`simd.shuffle` takes the new order as fixed lane numbers, known when compiling: `[4, 3, 2, 1]`";
                let AK::List(items) = &positional[1].kind else {
                    return err(Diagnostic::error(fixed).named("shuffle_not_fixed").at(positional[1].span));
                };
                let mut order = Vec::new();
                for it in items {
                    let AK::Int(text) = &it.kind else {
                        return err(Diagnostic::error(fixed).named("shuffle_not_fixed").at(it.span));
                    };
                    let n: u64 = text.parse().unwrap_or(u64::MAX);
                    if n < 1 || n > lanes as u64 {
                        return err(Diagnostic::error(format!("lane {} does not exist: the vector has {} lanes, 1 to {}", text, lanes, lanes)).named("index_out_of_range").at(it.span));
                    }
                    order.push(n as u32 - 1);
                }
                if order.len() != lanes as usize {
                    return err(Diagnostic::error(format!("the new order names {} lane(s), and the vector has {}: one lane number per lane", order.len(), lanes)).named("simd_lane_count_mismatch").at(positional[1].span));
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
                    Ty::Simd { elem, .. } => return err(Diagnostic::error(format!("`{}` folds a vector of answers, and this holds `{}` lanes: a comparison gives one (`$a < $b`)", name, self.spell(elem))).named("wrong_type").at(positional[0].span)),
                    other => return err(Diagnostic::error(format!("`{}` folds a vector of answers, and this is {}", name, crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                }
                (vec![Arg::Value(v)], Ty::Bool)
            }
            Builtin::SimdMax | Builtin::SimdMin | Builtin::SimdSum | Builtin::SimdSumLane | Builtin::SimdSumWrap => {
                arity(1, self)?;
                let v = self.expr(ctx, positional[0])?;
                let v = self.as_operand(ctx, v)?;
                let vt = self.resolve(&v.ty);
                let (lane, lanes) = match vt.base() {
                    Ty::Simd { elem, lanes } if elem.is_numeric() => ((**elem).clone(), *lanes),
                    Ty::Simd { elem, .. } => return err(Diagnostic::error(format!("`{}` folds a vector of numbers, and this holds `{}` lanes", name, self.spell(elem))).named("wrong_type").at(positional[0].span)),
                    other => return err(Diagnostic::error(format!("`{}` folds a vector of numbers, and this is {}", name, crate::types::a_type(&self.spell(other)))).named("wrong_type").at(positional[0].span)),
                };
                // `.wrap` wraps whole numbers, as `plus.wrap` does (design
                // claude's reading, 2026-10-07).
                if b == Builtin::SimdSumWrap && !lane.is_int() {
                    return err(Diagnostic::error(format!("`simd.sum.wrap` wraps whole numbers at the lane's type, and these lanes are `{}`: write `simd.sum`", self.spell(&lane))).named("needs_integer").at(span));
                }
                // The plain sum is a plain number, worked out in full and
                // checked where it lands (owner, 2026-10-07: "b").
                let ty = match b {
                    Builtin::SimdSum => Ty::simd_sum_exact(&lane, lanes).unwrap_or(lane),
                    _ => lane,
                };
                (vec![Arg::Value(v)], ty)
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
                self.items.structs.push(StructInfo { name: name.into(), path: name.into(), layout_c: false, opaque: false, fields: vec![FieldInfo { name: "key".into(), immut: true, ty: kt, bits: None }, FieldInfo { name: "value".into(), immut: true, ty: vt, bits: None }], union: false, blanks: Vec::new(), bounds: Vec::new(), span: Span::new(crate::source::FileId(0), 0, 0) });
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
                    (Verb::Lend, true) => err(Diagnostic::error(format!("`{}` changes the collection, so it takes `lend.mut`, not `lend`", name)).named("lend_mut_needed").at(e.span).fix_edit(e.span, Edit::Verb, "lend.mut")),
                    (Verb::LendMut, false) => {
                        let place = self.place(ctx, expr, PlaceUse::LendMut)?;
                        Ok(Arg::Lend { place, mutable: true })
                    }
                    (v, _) => err(Diagnostic::error(format!("`{}` looks at the collection: write `lend`{}, not `{}`", name, if mutates { ".mut" } else { "" }, v.spelled())).named("verb_wrong_for_parameter").at(e.span)),
                }
            }
            AK::Name(_) | AK::Field { .. } | AK::Index { .. } => {
                let place = self.place(ctx, e, PlaceUse::Read)?;
                let pt = self.resolve(&place.ty);
                let pn = self.place_name(ctx, &place);
                // A const is written bare (owner, 2026-10-10): nobody owns
                // it, so a look at it lends nothing away.
                let a_const = place.proj.is_empty() && ctx.locals[place.base.0 as usize].kind == LocalKind::ConstView;
                if pt.is_owned() && !(a_const && !mutates) {
                    return err(Diagnostic::error(format!("`{}` takes a lend: write `lend{} {}`", name, if mutates { ".mut" } else { "" }, pn)).named("owned_value_bare").at(e.span).fix_edit(e.span, Edit::Verb, if mutates { "lend.mut" } else { "lend" }));
                }
                Ok(Arg::Lend { place, mutable: mutates })
            }
            _ => {
                // a fresh value lives for the call
                let v = self.expr(ctx, e)?;
                if mutates {
                    return err(Diagnostic::error(format!("`{}` changes a collection you hold: `lend.mut` a variable", name)).named("lend_mut_needed").at(e.span).fix_edit(e.span, Edit::Verb, "lend.mut"));
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

    /// `pos i` where a position is wanted (owner, 2026-10-07: on every
    /// position).
    fn pos_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr, name: &str) -> Result<Arg> {
        let e = match &e.kind {
            AK::Pos(inner) => &**inner,
            _ if name.ends_with("slice") => return err(Diagnostic::error(format!("positions say `pos`: `{}(lend $xs, from: pos 2, to: pos 4)`", name)).named("position_unlabelled").at(e.span)),
            _ => return err(Diagnostic::error(format!("a position says `pos`: `{}(lend $xs, pos: 1)`", name)).named("position_unlabelled").at(e.span).fix_edit(e.span, Edit::Label, "pos: ")),
        };
        let v = self.expr(ctx, e)?;
        let vt = self.resolve(&v.ty);
        if !vt.is_int() {
            return err(Diagnostic::error(format!("a position is a whole number, and this is {}", crate::types::a_type(&self.spell(&vt)))).named("wrong_type").at(e.span));
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
        K::Binary { op, lhs, rhs, .. } => matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_)) && plain_group(lhs) && plain_group(rhs),
        K::Unary { op: UnOp::Neg, expr } => plain_group(expr),
        K::Int(_) | K::BigInt(_) => true,
        _ => false,
    }
}

/// Bare numbers — whole or with a point — with only `+ - x div` between
/// them: what a `deci` place makes decimals of.
/// The width of an exact literal (`*0.1`) not yet given by its place.
pub const EXACT_FREE: u16 = 0;

/// An exact literal no place gave a width: `d64`, the oracle's pick
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
            Err(crate::deci::LitErr::Digits(n)) => return err(Diagnostic::error(format!("`{}` has {} digits; d{} holds {}: write fewer digits, or use a wider deci", text, n, bits, p)).named("money::too_many_digits").at(e.span)),
            Err(crate::deci::LitErr::Range) => return err(Diagnostic::error(format!("`{}` is past what d{} holds: write fewer digits, or use a wider deci", text, bits)).named("money::too_many_digits").at(e.span)),
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
    // A percent stays one: its number is what settles (owner, 2026-10-17).
    e.ty = if percent_inner(&e.ty).is_some() { Ty::Percent(Box::new(Ty::Deci(bits))) } else { Ty::Deci(bits) };
    Ok(())
}

/// Every leaf a literal, every node arithmetic.
impl Checker {
    /// A group of bare numbers (not one literal) worked exactly, as wide as
    /// its own values need, where nothing else gives it a width: beside an
    /// exact answer, as a shift's count, in a comparison (check claude,
    /// seeds 27222, 27256, 27536). Left untyped it took 64 signed bits.
    fn bare_group_exact(&self, e: &mut Expr) {
        let t = self.resolve(&e.ty);
        // Untyped, or already exact at a limit its values pass.
        if literal_only(e) && !matches!(e.kind, K::Int(_)) && (matches!(t.base(), Ty::Int(IntW::Free(_))) || t.is_exact()) {
            if let Some(bits) = literal_group_bits(e) {
                let limit = bits.max(64).max(t.exact_limit().unwrap_or(0));
                retype_literal_arith(e, &Ty::exact_at(limit));
            }
        }
    }

    /// A shift still waiting for where its value goes (owner, 2026-10-07).
    fn width_pending(&self, t: &Ty) -> bool {
        matches!(t, Ty::Infer(_)) && self.widths_pending.iter().any(|(p, _, _)| self.resolve(p) == *t)
    }

    /// Whether raw memory or unproven code here needs `no_check` in front:
    /// in every Neuron, the marks being the Brainstem's since IEG joined it
    /// (owner, 2026-10-12).
    pub(super) fn needs_mark(&self, ctx: &FnCtx) -> bool {
        !ctx.tier.is_some_and(ast::Tier::trusts)
    }

    /// A struct's or enum's name with the Kit it comes from: `areas::Circle`.
    fn spell_with_kit(&self, t: &Ty) -> String {
        let path = match t.base() {
            Ty::Struct { id, .. } => self.items.structs[id.0 as usize].path.clone(),
            Ty::Enum { id, .. } => self.items.enums[id.0 as usize].path.clone(),
            _ => return self.spell(t),
        };
        let path = path.replace("::kit.", "::");
        path.strip_prefix("kit.").map(|p| format!("my.{}", p)).unwrap_or(path)
    }
}

/// One of std's names the oracle stands in for (owner, 2026-10-06: std
/// is `bare.kit_prefix.optional` for all), written without `std::`.
fn std_name(path: &ast::Path) -> bool {
    let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
    match segs.as_slice() {
        // `printu` and `printf` are std's old spellings, refused written
        // with `std::`; bare, `printf` is C's (design claude, 2026-10-19).
        ["print" | "format", ..] => true,
        ["exit" | "round" | "floor" | "ceil" | "trunc"] => true,
        ["round", _] => true,
        ["time" | "input", _, ..] => true,
        ["str", op] => matches!(*op, "split" | "find" | "contains" | "trim" | "upper" | "lower" | "replace" | "starts_with" | "ends_with" | "at" | "slice" | "join"),
        _ => false,
    }
}

/// What a bare std name is told outside std (owner, 2026-10-22), as the
/// oracle words it.
pub fn std_bare_message(path: &ast::Path) -> String {
    let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
    match segs.as_slice() {
        ["list", ..] => "`list` is std's: write `std::list` (`std::list[i64]`, `std::list.append(…)`), or name it for this file with `use std::list as list;` (owner, 2026-10-22)".into(),
        ["map", ..] => "`map` is std's: write `std::map` (`std::map[str.utf8, i32]`, `std::map.get(…)`), or name it for this file with `use std::map as map;` (owner, 2026-10-22)".into(),
        ["duration", ..] => "`duration` is std's: write `std::duration`, or `use std::duration as duration;` (owner, 2026-10-22)".into(),
        ["str", unit, ..] => format!("`str.{u}` is std's: write `std::str.{u}` (`std::str.each.grapheme(…)`), or name it for this file with `use std::str.{u} as str_{u};` (owner, 2026-10-22)", u = unit),
        _ => format!("`{w}` is std's: write `std::{w}`, or name it for this file with `use std::{h} as {h};` (owner, 2026-10-22)", w = path.spelled(), h = segs.first().copied().unwrap_or("")),
    }
}

impl Checker {
    /// Whether code in this module is std's own, where std's names are
    /// bare (owner, 2026-10-22: "own stays bare").
    pub fn in_std(&self, ctx: &FnCtx) -> bool {
        self.items.nerve_cord.is_some_and(|c| self.items.kit_root(ctx.module) == c)
    }
}

/// std's lists, maps and text units, which the language builds in, with
/// `std::` in front.
fn std_builtin(path: &ast::Path) -> bool {
    let segs: Vec<&str> = path.segs.iter().map(|s| s.name.as_str()).collect();
    match segs.as_slice() {
        // `std::map[…]`, a map's literal, as well as `std::map.get[…]`.
        ["list" | "map", ..] => true,
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
            matches!(op, B::Add | B::Sub | B::Mul | B::Pow | B::Div(_) | B::Mod(_) | B::Shl | B::Shr | B::AndBit | B::OrBit | B::Xor | B::AndBitWrap | B::OrBitWrap | B::XorWrap | B::WrapAdd | B::WrapSub | B::WrapMul | B::WrapPow) && literal_only(lhs) && literal_only(rhs)
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
/// The exact type a group of bare numbers is worked at: as wide as its
/// values need, at least 64 bits (check claude, seeds 28093, 28563).
/// A group of bare numbers (not one literal) worked exactly at its own
/// size, at least 64 bits.
fn exact_group(e: &mut Expr) {
    if literal_only(e) && !matches!(e.kind, K::Int(_)) && matches!(e.ty.base(), Ty::Int(IntW::Free(_))) {
        if let Some(bits) = literal_group_bits(e) {
            retype_literal_arith(e, &Ty::exact_at(bits.max(64)));
        }
    }
}

fn exact_for(e: &Expr) -> Ty {
    Ty::exact_at(literal_group_bits(e).unwrap_or(64).max(64))
}

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
                    BinOp::Div(r) | BinOp::DivIeee(r) => a.divmod(&b, *r)?.0,
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
/// 2026-09-24: a; `operators.md`): `'x' + 1.0` on an f32 is f32 math.
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
impl Checker {
    /// Two numeric families beside an operator: the side that converts
    /// without failing, to the other's type (an integer to a bin or a deci;
    /// a bin and a deci either way).
    fn mixed_families(&self, d: Diagnostic, (lt, ls): (&Ty, Span), (rt, rs): (&Ty, Span)) -> Diagnostic {
        let to = |t: &Ty| format!("to.{}", conv_spelling(&self.spell(t.base())));
        match (lt.base(), rt.base()) {
            (Ty::Int(_), Ty::Bin(_) | Ty::Deci(_)) => d.fix_edit(ls, Edit::Wrap, to(rt)),
            (Ty::Bin(_) | Ty::Deci(_), Ty::Int(_)) => d.fix_edit(rs, Edit::Wrap, to(lt)),
            (Ty::Bin(_), Ty::Deci(_)) | (Ty::Deci(_), Ty::Bin(_)) => d.fix_edit(rs, Edit::Wrap, to(lt)).fix_edit(ls, Edit::Wrap, to(rt)),
            _ => d,
        }
    }
}

/// A format text as a literal again: the line breaks and tabs it was read
/// with, escaped as written.
fn written_back(text: &str) -> String {
    format!("\"{}\"", text.replace('\n', "\\n").replace('\t', "\\t"))
}

pub(crate) fn conv_spelling(spelled: &str) -> String {
    match spelled {
        "integer" => "i32".to_string(),
        "bin" => "f64".to_string(),
        other => other.to_string(),
    }
}

fn article(s: &str) -> String {
    crate::types::a_type(s)
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


/// The cord calls Synapse builds (owner, 2026-10-09): the prints and
/// `exit`. A print goes to stdout unless it says `.stderr`.
pub const BUILT_CORD: [&str; 32] = [
    "exit",
    // The formatting print, and the text it makes (owner, 2026-09-22).
    "print.format", "print.format.stdout", "print.format.stderr", "format",
    // A print's own buffer, and sending what is held (owner, 2026-10-23);
    // its way when a print does not fit (2026-10-24).
    "print.writer", "print.writer.stderr", "print.flush",
    "print.writer.whole", "print.writer.whole.stderr", "print.writer.split", "print.writer.split.stderr",
    "print.written", "print.written.stdout", "print.written.stderr",
    "print.useful", "print.useful.stdout", "print.useful.stderr",
    "print.stored", "print.stored.stdout", "print.stored.stderr",
    "print.useful.auto_newline", "print.useful.auto_newline.stdout", "print.useful.auto_newline.stderr",
    "print.weird.useful", "print.weird.useful.stdout", "print.weird.useful.stderr",
    "print.weird.stored", "print.weird.stored.stdout", "print.weird.stored.stderr",
    "this_will_print_the_useful_numbers.this_will_go_to_the_standard_output",
    "this_will_print_the_useful_numbers.this_will_go_to_the_standard_error",
];

impl Checker {
    /// `incl.bytes[…]` / `incl.str.utf8[…]` / `incl.str.ascii[…]` (owner,
    /// 2026-10-10): the file read now, while building, from the Neuron's
    /// root whichever file says it; refused when it cannot be read or is
    /// not the text it is called, even where the code never runs. Text
    /// is a literal's, in the program file; bytes are a new `array`, an
    /// array being one that may change.
    fn incl(&mut self, ctx: &FnCtx, form: ast::InclForm, path: &str, path_span: Span, span: Span) -> Result<Expr> {
        let root = self.items.kit_root(ctx.module);
        let dir = self.items.neuron_dirs.get(&root).cloned().unwrap_or_else(|| std::path::PathBuf::from("."));
        let full = dir.join(path);
        let what = match form {
            ast::InclForm::Bytes => "incl.bytes".to_string(),
            ast::InclForm::Utf8 => "incl.str.utf8".to_string(),
            ast::InclForm::Ascii => "incl.str.ascii".to_string(),
            ast::InclForm::Utf16 { low_first } => format!("incl.str.utf16.{}", if low_first { "low_first" } else { "high_first" }),
            ast::InclForm::Utf32 { low_first } => format!("incl.str.utf32.{}", if low_first { "low_first" } else { "high_first" }),
        };
        let bytes = std::fs::read(&full).map_err(|e| Box::new(Diagnostic::error(format!("`{}` cannot read \"{}\" ({}, from the kit's root): {}", what, path, full.display(), e)).named("file_not_readable").at(path_span)))?;
        // Everything `incl` gives is a const's value only (owner,
        // 2026-10-10: "(c)" for the bytes, then "`a`" for the text).
        if !ctx.in_const {
            return err(Diagnostic::error(match form {
                ast::InclForm::Bytes => "`incl.bytes` is a constant: give it a name with `const $logo := incl.bytes[\"…\"];` (owner, 2026-10-10), and read it from there".to_string(),
                _ => format!("`incl.str.…` is a constant, as `incl.bytes` is: give it a name with `const $help := {}[\"…\"];` (owner, 2026-10-10), and read it from there", what),
            }).named("include_not_const").at(span));
        }
        match form {
            ast::InclForm::Bytes => {
                let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                let arg = Arg::Value(Expr { kind: K::Str(hex), ty: Ty::utf8(), span: path_span });
                Ok(Expr { kind: K::Call { callee: Callee::Builtin(Builtin::InclBytes), args: vec![arg], otherwise: None, fails: Vec::new() }, ty: Ty::Seq { elem: Box::new(Ty::int(false, 8)), kind: SeqKind::Array }, span })
            }
            ast::InclForm::Utf8 => match String::from_utf8(bytes) {
                Ok(text) => Ok(Expr { kind: K::Str(text), ty: Ty::utf8(), span }),
                Err(e) => err(Diagnostic::error(format!("\"{}\" is not valid UTF-8 text (byte {} is not), so `incl.str.utf8` cannot hold it; `incl.bytes` takes any file", path, e.utf8_error().valid_up_to() + 1)).named("included_text_invalid").at(path_span)),
            },
            ast::InclForm::Ascii => match bytes.iter().position(|b| !b.is_ascii()) {
                Some(i) => err(Diagnostic::error(format!("\"{}\" is not ASCII text (byte {} is {}), so `incl.str.ascii` cannot hold it; `incl.str.utf8` takes UTF-8 and `incl.bytes` any file", path, i + 1, bytes[i])).named("included_text_invalid").at(path_span)),
                None => Ok(Expr { kind: K::Str(String::from_utf8(bytes).unwrap()), ty: Ty::Str(crate::types::Enc::Ascii), span }),
            },
            // In the byte order it says (owner, 2026-10-10); a byte-order
            // mark that agrees is dropped, one that disagrees refused.
            ast::InclForm::Utf16 { low_first } | ast::InclForm::Utf32 { low_first } => {
                let (unit, enc, ty) = match form {
                    ast::InclForm::Utf16 { .. } => (2usize, "str.utf16", Ty::Str(crate::types::Enc::Utf16)),
                    _ => (4usize, "str.utf32", Ty::Str(crate::types::Enc::Utf32)),
                };
                if bytes.len() % unit != 0 {
                    return err(Diagnostic::error(format!("\"{}\" is {} bytes, not a whole number of {}-byte units, so it is not {} text", path, bytes.len(), unit, enc)).named("included_text_invalid").at(path_span));
                }
                let mut units: Vec<u32> = bytes
                    .chunks(unit)
                    .map(|c| {
                        let mut v: u32 = 0;
                        for k in 0..unit {
                            let b = if low_first { c[unit - 1 - k] } else { c[k] };
                            v = v << 8 | b as u32;
                        }
                        v
                    })
                    .collect();
                let flipped = if unit == 2 { 0xFFFE } else { 0xFFFE_0000 };
                match units.first() {
                    Some(0xFEFF) => {
                        units.remove(0);
                    }
                    Some(u) if *u == flipped => {
                        let said = |low: bool| if low { "low byte first" } else { "high byte first" };
                        return err(Diagnostic::error(format!("\"{}\" starts with a byte-order mark saying {}, but `{}` says {}", path, said(!low_first), what, said(low_first))).named("included_text_invalid").at(path_span));
                    }
                    _ => {}
                }
                let bad = || Box::new(Diagnostic::error(format!("\"{}\" is not valid {} text in that byte order, so `{}` cannot hold it; `incl.bytes` takes any file", path, enc, what)).named("included_text_invalid").at(path_span));
                let text: String = if unit == 2 {
                    let u16s: Vec<u16> = units.iter().map(|u| *u as u16).collect();
                    char::decode_utf16(u16s).collect::<std::result::Result<String, _>>().map_err(|_| bad())?
                } else {
                    units.iter().map(|u| char::from_u32(*u)).collect::<Option<String>>().ok_or_else(bad)?
                };
                Ok(Expr { kind: K::Str(text), ty, span })
            }
        }
    }

    /// Testing aid: `std::` typed as the oracle emulates it. Not std.
    /// A print's or a flush's `lend.mut $out`: a writer, or refused, a
    /// print never changing what it prints (owner, 2026-10-23).
    fn writer_arg(&mut self, ctx: &mut FnCtx, e: &ast::Expr) -> Result<Arg> {
        let shown = match &e.kind {
            AK::Verb { expr, .. } => match &expr.kind {
                AK::Name(q) => format!("lend.mut ${}", q.name),
                _ => "lend.mut …".to_string(),
            },
            _ => "lend.mut $out".to_string(),
        };
        let is_writer = match &e.kind {
            AK::Verb { verb: Verb::LendMut, expr, .. } => match &expr.kind {
                AK::Name(q) => ctx.lookup(&q.name).is_some_and(|id| matches!(self.resolve(&ctx.locals[id.0 as usize].ty).base(), Ty::Writer)),
                _ => false,
            },
            _ => false,
        };
        if !is_writer {
            return err(Diagnostic::error(format!("a print never changes what it prints, so `{}` names a writer to print into: `let mut $out := std::print.writer(size: 4096 bytes);`", shown)).named("print_into_non_writer").at(e.span));
        }
        self.lent_arg(ctx, e, "a print", true)
    }

    /// A format call's text and values (`examples/print_format.cy`).
    fn check_format(&self, ctx: &FnCtx, values: &[&ast::Expr], checked: &[Arg], span: Span) -> Result<()> {
        let Some(text_e) = values.first() else {
            return err(Diagnostic::error("a format call starts with its text: `std::print.format(\"{} items\\n\", $n)`").named("argument_missing").at(span));
        };
        let AK::Str(text) = &text_e.kind else {
            if ctx.tier.is_some() {
                return err(Diagnostic::error("not yet: a format text known only when the program runs").named("not_built_yet").at(text_e.span));
            }
            return err(Diagnostic::error("a format text not written in the call is read only when it runs, so its holes cannot be checked now: that is a no_check site").named("no_check_missing").at(text_e.span));
        };
        let pieces = crate::format::parse(text).map_err(|why| {
            let d = Diagnostic::error(why).named("format_text_wrong").at(text_e.span);
            Box::new(match crate::format::repair(text) {
                Some(fixed) => d.fix_edit(text_e.span, Edit::Replace, written_back(&fixed)),
                None => d,
            })
        })?;
        let given = values.len() - 1;
        let (most, unused) = crate::format::wanted(&pieces);
        if most > given {
            return err(Diagnostic::error(format!("the format text has more holes than values: it asks for {}, and {} {} given", most, given, if given == 1 { "is" } else { "are" })).named("format_holes_mismatch").at(span));
        }
        if most < given {
            return err(Diagnostic::error(format!("the format text has fewer holes than values: {} given, and it asks for {}", given, most)).named("format_holes_mismatch").at(span));
        }
        if let Some(n) = unused.first() {
            return err(Diagnostic::error(format!("value {} is never used: the format text has no `{{{}}}`", n + 1, n + 1)).named("format_holes_mismatch").at(span));
        }
        for p in &pieces {
            let crate::format::Piece::Hole(i, spec) = p else { continue };
            let Some(a) = checked.get(i + 1) else { continue };
            let t = self.resolve(&self.arg_type(a));
            let kind_of = |t: &Ty| -> &'static str {
                match t.base() {
                    Ty::Str(_) => "str",
                    Ty::Int(_) => "integer",
                    Ty::Bin(_) | Ty::Brain => "bin",
                    Ty::Deci(_) => "deci",
                    _ => "other",
                }
            };
            let k = kind_of(&t);
            let shown = if k == "str" { "a str".to_string() } else { crate::types::a_type(&self.spell(&t)) };
            let wants = |ok: &[&str], what: &str| -> Result<()> {
                if ok.contains(&k) {
                    return Ok(());
                }
                let d = Diagnostic::error(format!("{} takes {} and does not apply to {}", what, if ok == ["integer"] { "a whole number" } else { "a bin or a deci" }, shown)).named("format_text_wrong").at(values[i + 1].span);
                err(match crate::format::without_kind(text, *i) {
                    Some(fixed) if spec.places.is_none() => d.fix_edit(text_e.span, Edit::Replace, written_back(&fixed)),
                    _ => d,
                })
            };
            match spec.kind {
                Some(c @ ('x' | 'X' | 'o' | 'b')) => wants(&["integer"], &format!("`{{:{}}}`", c))?,
                Some('e') => wants(&["bin", "deci"], "`{:e}`")?,
                _ => {}
            }
            if let (Some(n), None | Some('e')) = (spec.places, spec.kind) {
                if spec.kind.is_none() {
                    wants(&["bin", "deci"], &format!("`{{:.{}}}`", n))?;
                }
            }
        }
        Ok(())
    }

    fn std_stub(&mut self, ctx: &mut FnCtx, path: &ast::Path, args: &[AArg], otherwise: Option<&ast::Otherwise>, span: Span, used: bool) -> Result<Expr> {
        let name = path.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(".");
        let labelled;
        let args = if name.starts_with("str.slice.") {
            labelled = slice_args(&format!("std::{}", name), args, span)?;
            &labelled[..]
        } else {
            args
        };
        let mut positional = Vec::new();
        for a in args {
            match a {
                AArg::Positional(e) => positional.push(e),
                _ => return err(Diagnostic::error("std takes values in order").named("argument_label_unknown").at(a.span())),
            }
        }
        let mut hargs = Vec::new();
        let mut fails: Vec<(String, Option<Ty>)> = Vec::new();
        // `otherwise std::print.useful` (or `.stored`) says how a
        // `print.written` prints a value it has no written form for (owner,
        // 2026-10-17); no other print takes one.
        let print_as = match otherwise {
            Some(ast::Otherwise::Value(ast::Expr { kind: AK::Path(p), .. })) if p.kit.as_ref().is_some_and(|k| k.name == "std") && p.segs.len() == 2 && p.segs[0].name == "print" && matches!(p.segs[1].name.as_str(), "useful" | "stored") => Some(p.segs[1].name.clone()),
            _ => None,
        };
        let written = name == "print.written" || name.starts_with("print.written.");
        if print_as.is_some() && !written {
            return err(Diagnostic::error(format!("`otherwise std::print.…` says how a `print.written` prints a value it has no written form for; `std::{}` prints every value already: drop it", name)).named("print_form_unsaid").at(span));
        }
        if written {
            // Each value as its literal was typed; the rest as the print's
            // `otherwise` says, which is then the print that runs.
            let stream = name.strip_prefix("print.written").unwrap_or("").to_string();
            for e in &positional {
                match self.written_form(ctx, e) {
                    Some(text) => hargs.push(Arg::Value(Expr { kind: K::Str(text), ty: Ty::utf8(), span: e.span })),
                    None => {
                        if print_as.is_none() {
                            return err(Diagnostic::error("this was not typed as a literal here, so `print.written` has no written form of it: say how it prints instead, `otherwise std::print.useful` (or `.stored`)").named("written_not_literal").at(e.span));
                        }
                        let v = self.expr(ctx, e)?;
                        let v = self.as_operand(ctx, v)?;
                        hargs.push(Arg::Value(v));
                    }
                }
            }
            let run = format!("print.{}{}", print_as.as_deref().unwrap_or("useful"), stream);
            return Ok(Expr { kind: K::Call { callee: Callee::Builtin(Builtin::Std(run)), args: hargs, otherwise: None, fails: Vec::new() }, ty: Ty::Nothing, span });
        }
        let ty = match name.as_str() {
            // A print goes to stdout unless it says `.stderr` (owner,
            // 2026-10-07); `.stdout` may still be written. `format[…]` makes
            // the text `print.format` would print ("`a`").
            // `std::print.writer(size: 4096 bytes)` (owner, 2026-10-23: "Just
            // `bytes` for now").
            "print.writer" | "print.writer.stderr" | "print.writer.whole" | "print.writer.whole.stderr" | "print.writer.split" | "print.writer.split.stderr" => {
                // `spellings.cyr` lists the call, so `size:` is taken out
                // before this reads it.
                let size = match positional.as_slice() {
                    [e] => {
                        let v = match &e.kind {
                            AK::Label(w, v) if w.name == "size" => &**v,
                            _ => *e,
                        };
                        match &v.kind {
                            AK::Duration { value, unit } if unit.name == "bytes" => value.replace('_', ""),
                            _ => return err(Diagnostic::error("a writer's size says its unit: `size: 4096 bytes`").named("size_unit_unsaid").at(v.span).fix_edit(crate::source::Span::new(v.span.file, v.span.end as usize, v.span.end as usize), Edit::Insert, " bytes")),
                        }
                    }
                    _ => return err(Diagnostic::error(format!("a writer says its size, and only that: `std::{}(size: 4096 bytes)`", name)).named("wrong_argument_count").at(span)),
                };
                let n: i128 = size.parse().map_err(|_| Box::new(Diagnostic::error("a writer's size is a whole number of bytes").named("wrong_type").at(span)))?;
                if n < 1 {
                    return err(Diagnostic::error("a writer holds at least 1 byte").named("writer_size_zero").at(span));
                }
                hargs.push(Arg::Value(Expr { kind: K::Int(n), ty: Ty::u64(), span }));
                Ty::Writer
            }
            // `std::print.flush()`: what kayo holds, now; `std::print.flush(lend.mut
            // $out)`: a writer's (owner, 2026-10-23).
            "print.flush" => {
                match positional.as_slice() {
                    [] => {}
                    [w] => {
                        let a = self.writer_arg(ctx, w)?;
                        hargs.push(a);
                        if otherwise.is_some() {
                            fails.push(("write_failed".into(), None));
                        }
                    }
                    _ => return err(Diagnostic::error("`std::print.flush` sends what kayo holds, `std::print.flush()`, or a writer's, `std::print.flush(lend.mut $out)`").named("wrong_argument_count").at(span)),
                }
                Ty::Nothing
            }
            "format" | "print.stored" | "print.useful" | "print.weird.stored" | "print.weird.useful" | "print.useful.auto_newline" | "print.format" | "print.stored.stdout" | "print.stored.stderr" | "print.useful.stdout" | "print.useful.stderr" | "print.weird.stored.stdout" | "print.weird.stored.stderr" | "print.weird.useful.stdout" | "print.weird.useful.stderr" | "print.useful.auto_newline.stdout" | "print.useful.auto_newline.stderr" | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_output" | "this_will_print_the_useful_numbers.this_will_go_to_the_standard_error" | "print.format.stdout" | "print.format.stderr" => {
                // `lend.mut $out` first: the writer it goes into (owner,
                // 2026-10-23); a print never changes what it prints.
                let mut values = &positional[..];
                if let Some(first) = positional.first() {
                    if matches!(&first.kind, AK::Verb { verb: Verb::LendMut, .. }) && name != "format" {
                        let a = self.writer_arg(ctx, first)?;
                        hargs.push(a);
                        values = &positional[1..];
                    }
                }
                for e in values {
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
                // A format text, read now (owner, 2026-09-22; `{}` holes
                // 2026-10-01): its holes against the values, each hole's kind
                // against its value's type.
                if name == "format" || name.starts_with("print.format") {
                    let offset = hargs.len() - values.len();
                    self.check_format(ctx, values, &hargs[offset..], span)?;
                }
                // A failed write is handled where the print is (owner,
                // 2026-10-23: "handle with `otherwise`").
                if otherwise.is_some() && name != "format" {
                    fails.push(("write_failed".into(), None));
                }
                if name == "format" { Ty::utf8() } else { Ty::Nothing }
            }
            "exit" => {
                if positional.len() != 1 {
                    return err(Diagnostic::error("`std::exit` takes one value, the exit code").named("wrong_argument_count").at(span));
                }
                let v = self.expr(ctx, positional[0])?;
                let v = self.expect_store(ctx, v, &Ty::int(false, 8), "the exit code")?;
                hargs.push(Arg::Value(v));
                Ty::Nothing
            }
            "input.line" | "input.all" => {
                if !positional.is_empty() {
                    return err(Diagnostic::error(format!("`std::{}` takes no arguments", name)).named("wrong_argument_count").at(span));
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
            // std's rounding family (owner, 2026-10-17): `round.near`,
            // `half_up`, `floor`, `ceil`, `trunc`, on a bin or a deci, to a
            // whole number or to `places: N`; the answer is the kind that
            // went in.
            _ if name.starts_with("round.") => {
                let way = &name[6..];
                if !matches!(way, "near" | "half_up" | "floor" | "ceil" | "trunc") {
                    return err(Diagnostic::error(format!("`std::{}` is not a way to round: `round.near`, `round.half_up`, `round.floor`, `round.ceil` or `round.trunc`", name)).named("unknown_rounding").at(span));
                }
                let Some(first) = positional.first() else {
                    return err(Diagnostic::error(format!("`std::{}` takes the number to round: `std::{}($x)`", name, name)).named("wrong_argument_count").at(span));
                };
                let a = self.expr(ctx, first)?;
                let a = self.as_operand(ctx, a)?;
                let t = self.resolve(&a.ty);
                if !matches!(t.base(), Ty::Bin(_) | Ty::Deci(_) | Ty::Brain) {
                    return err(Diagnostic::error(format!("std::{} takes a bin or a deci, got {}", name, crate::types::a_type(&self.spell(&t)))).named("wrong_type").at(first.span));
                }
                hargs.push(Arg::Value(a));
                if let Some(d) = positional.get(1) {
                    if let AK::Unary { op: ast::UnOp::Neg, .. } = &d.kind {
                        return err(Diagnostic::error("`places:` is a whole number, 0 or more").named("count_negative").at(d.span));
                    }
                    let n = self.expr(ctx, d)?;
                    let n = self.as_operand(ctx, n)?;
                    if !self.resolve(&n.ty).is_int() {
                        return err(Diagnostic::error("`places:` is a whole number, 0 or more").named("wrong_type").at(d.span));
                    }
                    hargs.push(Arg::Value(n));
                }
                t
            }
            "c.errno" => Ty::int(true, 32),
            _ if name.starts_with("str.") => {
                // `str.at.X` and `str.slice.X` say each position with `pos`
                // (owner, 2026-10-07: "`a`, if it's correct").
                let positions = name.starts_with("str.at.") || name.starts_with("str.slice.");
                for (k, e) in positional.iter().enumerate() {
                    let e: &ast::Expr = match (&e.kind, positions && k >= 1) {
                        (AK::Pos(inner), true) if written_below_one(inner) => return err(Diagnostic::error(NO_POSITION).named("index_out_of_range").at(inner.span)),
                        (AK::Pos(inner), true) => inner,
                        (_, true) => return err(Diagnostic::error(format!("a position says `pos`: `std::{}(lend $s, pos: 3)`", name)).named("position_unlabelled").at(e.span).fix_edit(e.span, Edit::Label, "pos: ")),
                        _ => e,
                    };
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
                    // A `u8`, as std's own `str.at.byte` gives it
                    // (a022de88).
                    "at.byte" => {
                        fails.push(("out_of_range".into(), None));
                        Ty::int(false, 8)
                    }
                    "slice.grapheme" | "slice.point" => {
                        fails.push(("out_of_range".into(), None));
                        Ty::utf8()
                    }
                    other => return err(Diagnostic::error(format!("`std::str.{}` does not exist", other)).named("unknown_std_name").at(span)),
                }
            }
            _ => return err(Diagnostic::error(format!("`std::{}` does not exist", name)).named("unknown_std_name").at(span)),
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
            err(Diagnostic::error(format!("`.{}` follows the verb: `to.{}.{}(…)` (owner, 2026-09-30)", f, f, rest.join("."))).named("conversion_form").at(target.span))
        }
        ast::TypeKind::Path { path, args } if path.segs.len() >= 2 && (path.segs[0].name == "useful" || path.segs[0].name == "stored") => {
            let mut p = path.clone();
            p.segs.remove(0);
            Ok((Some(path.segs[0].name == "useful"), Some(ast::Type { kind: ast::TypeKind::Path { path: p, args: args.clone() }, span: target.span })))
        }
        ast::TypeKind::Bound { inner, lo, hi, unit, span, old, zero, places } => {
            let (form, inner2) = conv_form(inner)?;
            Ok((form, inner2.map(|i| ast::Type { kind: ast::TypeKind::Bound { inner: Box::new(i), lo: lo.clone(), hi: hi.clone(), unit: unit.clone(), span: *span, old: old.clone(), zero: *zero, places: *places }, span: target.span })))
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
        // Every rounding of a division too, and what it leaves over: a
        // brain16 takes them all (`div.trunc` beside `div.near`).
        K::Binary { op: BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_), lhs, rhs, .. } => bare_number_group(lhs) && bare_number_group(rhs),
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

/// A slice's arguments as the checker reads them (owner, 2026-10-12): the
/// collection, then `from pos 2` and `to pos 4`, or `before pos 5`, which is
/// `to pos 5 minus 1` — so a slice ending before where it starts is empty,
/// as one whose last is one before its first is.
fn slice_args(name: &str, args: &[AArg], span: Span) -> Result<Vec<AArg>> {
    let how = format!("a slice says where it runs: `{}(lend $xs, from: pos 2, to: pos 4)`, or `before: pos 5` to leave 5 out (owner, 2026-10-12)", name);
    if args.len() != 3 {
        if args.iter().skip(1).any(|a| matches!(a, AArg::Positional(e) if matches!(e.kind, AK::Pos(_)))) {
            return err(Diagnostic::error(how).named("old_spelling").at(span));
        }
        return Ok(args.to_vec());
    }
    // `pos: 1, pos: 2`: each side says which it is.
    // Where a side's word goes and its value starts, `pos: 1` labelled or
    // read as a position.
    let side = |a: &AArg| match a {
        AArg::Named(l, x) if l.name == "pos" => Some((l.span.start, x.span.start)),
        AArg::Positional(e) => match &e.kind {
            AK::Pos(inner) => Some((e.span.start, inner.span.start)),
            _ => None,
        },
        _ => None,
    };
    let labelled = |d: Diagnostic| match (side(&args[1]), side(&args[2])) {
        (Some((a, x)), Some((b, y))) => d
            .fix_edit(Span::new(span.file, a as usize, x as usize), Edit::Replace, "from: pos ")
            .fix_also(Span::new(span.file, b as usize, y as usize), Edit::Replace, "to: pos "),
        _ => d,
    };
    let first = match &args[1] {
        AArg::Named(l, e) if l.name == "from" => e.clone(),
        AArg::Positional(e) if !matches!(e.kind, AK::Pos(_)) => return err(Diagnostic::error(format!("a slice starts `from: pos …`: `{}(lend $xs, from: pos 2, to: pos 4)`", name)).named("position_unlabelled").at(e.span)),
        other => return err(labelled(Diagnostic::error(how.clone()).named("old_spelling").at(other.span()))),
    };
    let last = match &args[2] {
        AArg::Named(l, e) if l.name == "to" => e.clone(),
        AArg::Named(l, e) if l.name == "before" => match &e.kind {
            AK::Pos(inner) => {
                let one = ast::Expr { kind: AK::Int("1".into()), span: inner.span };
                let less = ast::Expr { kind: AK::Binary { op: BinOp::Sub, op_span: l.span, lhs: inner.clone(), rhs: Box::new(one) }, span: inner.span };
                ast::Expr { kind: AK::Pos(Box::new(less)), span: e.span }
            }
            _ => e.clone(),
        },
        other => return err(Diagnostic::error(how).named("old_spelling").at(other.span())),
    };
    Ok(vec![args[0].clone(), AArg::Positional(first), AArg::Positional(last)])
}

impl Checker {
    /// What a whole number can be, worked out from its operands (owner,
    /// 2026-10-12: "(a)"): a literal is itself, a typed value its type's
    /// range, a `:=` local the range it was given, and `plus`, `minus`,
    /// `times`, `div.X`, `mod.X` and negation what their operands allow;
    /// `None` where nothing bounds it.
    pub(crate) fn int_span(&self, ctx: &FnCtx, e: &Expr) -> Option<(i128, i128)> {
        let typed = |t: &Ty| match self.resolve(t).base() {
            t @ Ty::Int(IntW::Fixed { signed, bits }) if !t.is_exact() => Some(crate::resolve::int_range(*signed, *bits)),
            _ => None,
        };
        match &e.kind {
            K::Int(v) => Some((*v, *v)),
            K::Local(id) => ctx.int_ranges.get(id).copied().or_else(|| typed(&ctx.locals[id.0 as usize].ty)),
            K::Unary { op: UnOp::Neg, expr } => {
                let (a, b) = self.int_span(ctx, expr)?;
                Some((b.checked_neg()?, a.checked_neg()?))
            }
            K::Binary { op, lhs, rhs, .. } => {
                let (a, b) = self.int_span(ctx, lhs)?;
                let (c, d) = self.int_span(ctx, rhs)?;
                match op {
                    BinOp::Add => Some((a.checked_add(c)?, b.checked_add(d)?)),
                    BinOp::Sub => Some((a.checked_sub(d)?, b.checked_sub(c)?)),
                    BinOp::Mul => {
                        let p = [a.checked_mul(c)?, a.checked_mul(d)?, b.checked_mul(c)?, b.checked_mul(d)?];
                        Some((*p.iter().min()?, *p.iter().max()?))
                    }
                    // A quotient is no bigger than what is divided.
                    BinOp::Div(_) => {
                        let m = i128::try_from(a.unsigned_abs().max(b.unsigned_abs())).ok()?;
                        Some((-m, m))
                    }
                    // A remainder is smaller than the divisor.
                    BinOp::Mod(_) => {
                        let m = i128::try_from(c.unsigned_abs().max(d.unsigned_abs())).ok()?;
                        Some((-m, m))
                    }
                    _ => typed(&e.ty),
                }
            }
            _ => typed(&e.ty),
        }
    }
}

/// A text position is `u64[min 1]` (owner, 2026-10-17: "(a)"): a written
/// one of 0 or below is refused before the program runs.
const NO_POSITION: &str = "0 is no position: a position is `u64[min: 1]`, and `pos 1` is the first";

/// Whether a written position is a number below 1: `0`, `-3`.
fn written_below_one(e: &ast::Expr) -> bool {
    match &e.kind {
        AK::Int(t) => t.trim_start_matches('0').is_empty(),
        AK::Unary { op: ast::UnOp::Neg, expr } => matches!(&expr.kind, AK::Int(_)),
        AK::Paren(inner) => written_below_one(inner),
        _ => false,
    }
}

/// A literal as it was typed, for `print.written` (owner, 2026-10-17): its
/// digits (a percent's in `%`, its trailing zeros kept), its sign, its text.
pub fn literal_written(e: &ast::Expr) -> Option<String> {
    match &e.kind {
        AK::Int(t) | AK::Bin(t) => Some(t.replace('_', "")),
        AK::Exact(t) => Some(t.replace('_', "")),
        AK::Percent(lit) => {
            let t = match &lit.kind {
                AK::Bin(t) | AK::Exact(t) => t.replace('_', ""),
                _ => return None,
            };
            // The places it was written with: the fraction's, less two.
            let places = t.split_once('.').map_or(0, |(_, f)| f.len()).saturating_sub(2);
            let mut shown = crate::types::percent_text(&t);
            let have = shown.split_once('.').map_or(0, |(_, f)| f.len());
            if have < places {
                if have == 0 {
                    shown.push('.');
                }
                shown.push_str(&"0".repeat(places - have));
            }
            Some(format!("{}%", shown))
        }
        AK::Unary { op: UnOp::Neg, expr } => literal_written(expr).map(|t| format!("-{}", t)),
        AK::Typed { value, .. } | AK::Paren(value) => literal_written(value),
        AK::Bool(b) => Some(b.to_string()),
        AK::Str(s) => Some(s.clone()),
        AK::Newline => Some("\n".to_string()),
        _ => None,
    }
}

impl Checker {
    /// What `print.written` prints of an argument: a literal as typed, or a
    /// name declared with one and never changeable; `None` for anything
    /// else, which has no written form.
    fn written_form(&self, ctx: &FnCtx, e: &ast::Expr) -> Option<String> {
        match &e.kind {
            AK::Name(q) => ctx.lookup(&q.name).and_then(|id| ctx.written.get(&id).cloned()),
            _ => literal_written(e),
        }
    }
}

/// An argument as it is written, for picking a version.
enum Peek {
    Ty(Ty),
    /// A variable whose width is not settled yet: any of its family.
    Free(Ty),
    Int(String),
    Bin(String),
    Deci(String),
    Text,
    Unknown,
}

/// Whether a place holds a plain number, not a percent: a `%` literal in
/// its value is the number over 100 (owner, 2026-10-17).
pub fn plain_number(t: &Ty) -> bool {
    percent_inner(t).is_none() && matches!(t.base(), Ty::Bin(_) | Ty::Deci(_) | Ty::Brain)
}

/// The most places a type keeps (`max_places`), through what wraps it.
pub fn places_max(t: &Ty) -> Option<u32> {
    match t {
        Ty::Places { max, inner, .. } => max.or_else(|| places_max(inner)),
        Ty::Bounded { inner, .. } | Ty::Percent(inner) => places_max(inner),
        _ => None,
    }
}

/// The fewest places a type prints (`min_places`), through what wraps it.
pub fn places_min(t: &Ty) -> Option<u32> {
    match t {
        Ty::Places { min, inner, .. } => min.or_else(|| places_min(inner)),
        Ty::Bounded { inner, .. } | Ty::Percent(inner) => places_min(inner),
        _ => None,
    }
}

/// How many places a value can have, as a store into `max_places` asks.
pub enum PlacesOf {
    /// A literal's, as written.
    Literal(u32),
    /// At most this many.
    Can(u32),
    Unknown,
}

/// A percent's number (owner, 2026-10-17), through a bound around it.
pub fn percent_inner(t: &Ty) -> Option<Ty> {
    match t {
        Ty::Percent(inner) => Some((**inner).clone()),
        Ty::Bounded { inner, .. } | Ty::Places { inner, .. } => percent_inner(inner),
        _ => None,
    }
}

/// Whether an argument fits a version's parameter of type `t`.
fn peek_fits(k: &Peek, t: &Ty) -> bool {
    match k {
        // A text whose encoding is not settled yet fits any text.
        Peek::Ty(a) => a.base() == t || (matches!(a.base(), Ty::Str(crate::types::Enc::Free(_))) && matches!(t, Ty::Str(_))),
        Peek::Free(Ty::Int(_)) => matches!(t, Ty::Int(_)),
        Peek::Free(_) => matches!(t, Ty::Bin(_)),
        Peek::Int(v) => match t {
            Ty::Int(IntW::Fixed { signed, bits }) => match v.parse::<i128>() {
                Ok(n) => {
                    let (lo, hi) = crate::bigint::type_range(*signed, *bits);
                    let n = crate::bigint::Big::from_i128(n);
                    n.sub(&lo).is_neg() == false && hi.sub(&n).is_neg() == false
                }
                Err(_) => *bits > 128,
            },
            Ty::Int(_) | Ty::Bin(_) | Ty::Brain => true,
            _ => false,
        },
        Peek::Bin(_) => matches!(t, Ty::Bin(_) | Ty::Brain),
        // `exactly 2.665` fits each deci whose digits hold it.
        Peek::Deci(v) => matches!(t, Ty::Deci(b) if crate::deci::literal(v, *b).is_ok()),
        Peek::Text => matches!(t, Ty::Str(_)),
        Peek::Unknown => true,
    }
}

