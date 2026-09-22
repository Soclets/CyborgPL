//! The checker: from the parsed Limb to `hir` — every name resolved,
//! every expression typed, every call bound, every verb agreed with its
//! parameter, every failure handled. Ownership *flow* (a value given
//! twice, used after `give`, a lend outliving its owner) is MIR's job;
//! the checker settles what each spelling *means*.
//!
//! Functions are checked per **instance**: one source function, one
//! shape (generic blanks filled from the call, a variadic bundle typed by
//! its arguments) — the per-shape compilation `generics.md` decides.

pub mod expr;
pub mod stmt;

use std::collections::HashMap;

use crate::ast::{self, Mutability, Tier};
use crate::diag::{err, Diagnostic, Result};
use crate::hir::{self, FnInst, Local, LocalId, LocalKind};
use crate::resolve::{FnId, Items, ModuleId, ROOT};
use crate::source::Span;
use crate::types::*;

/// One instance to check: a function and its shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstKey {
    pub fn_id: FnId,
    pub blanks: Vec<Ty>,
    /// The variadic bundle's element types, if the function has one.
    pub bundle: Option<Vec<Ty>>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub key: InstKey,
    pub sig: hir::Signature,
    pub body: Option<hir::Body>,
    /// The instance is a function literal: its captures are extra leading locals.
    pub captures: Vec<(String, ast::Verb, Ty)>,
}

/// The checked program.
pub struct Program {
    pub items: Items,
    pub instances: Vec<Instance>,
    pub main: Option<FnInst>,
    /// Consts, evaluated in order before MAIN.
    pub const_values: Vec<hir::Expr>,
    pub notes: Vec<Diagnostic>,
    /// `&` spellings seen: one remark per occurrence (owner, 2026-09-21).
    pub amp_count: usize,
}

pub struct Checker {
    pub items: Items,
    pub instances: Vec<Instance>,
    inst_index: HashMap<InstKey, FnInst>,
    /// Instances whose bodies still wait to be checked.
    pending: Vec<FnInst>,
    /// Inference variables: what each `Infer` has been unified with.
    pub subst: HashMap<InferId, Ty>,
    next_infer: u32,
    pub const_values: Vec<Option<hir::Expr>>,
    pub notes: Vec<Diagnostic>,
    pub amp_count: usize,
    /// Anonymous `fn` literals get a synthetic FnId each.
    literal_fns: Vec<(ast::FnLit, ModuleId)>,
    /// A testing aid only: type `std::` calls as the oracle emulates them, so the
    /// checker can be run over the examples before std exists. Off by default;
    /// nothing is generated for them.
    pub std_stubs: bool,
}

/// The state while checking one body.
pub struct FnCtx {
    pub inst: FnInst,
    pub module: ModuleId,
    pub locals: Vec<Local>,
    /// Scopes: name → local, innermost last.
    pub scopes: Vec<HashMap<String, LocalId>>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
    pub loop_depth: u32,
    pub is_main: bool,
    pub is_task: bool,
    pub name: String,
    /// Names captured by an enclosing fn literal are visible only through `with`.
    pub captured_outer: Option<Vec<(String, ast::Verb, Ty, LocalId)>>,
    pub tier: Option<Tier>,
    /// A generic instance: the blanks that were filled, by name, for messages.
    pub blank_names: Vec<String>,
    /// Locals holding a list literal whose length is still known (for `...`).
    pub literal_lengths: HashMap<LocalId, usize>,
}

impl FnCtx {
    pub fn lookup(&self, name: &str) -> Option<LocalId> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }
    pub fn declare(&mut self, name: &str, local: Local) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(local);
        self.scopes.last_mut().unwrap().insert(name.to_string(), id);
        id
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
    /// Is `name` declared in an *enclosing* scope (not the innermost)?
    pub fn visible_outside(&self, name: &str) -> Option<LocalId> {
        for scope in self.scopes.iter().rev().skip(1) {
            if let Some(id) = scope.get(name) {
                return Some(*id);
            }
        }
        None
    }
    pub fn in_current_scope(&self, name: &str) -> Option<LocalId> {
        self.scopes.last().unwrap().get(name).copied()
    }
}

impl Checker {
    pub fn new(items: Items) -> Checker {
        let n = items.consts.len();
        Checker { items, instances: Vec::new(), inst_index: HashMap::new(), pending: Vec::new(), subst: HashMap::new(), next_infer: 1, const_values: vec![None; n], notes: Vec::new(), amp_count: 0, literal_fns: Vec::new(), std_stubs: false }
    }

    pub fn fresh(&mut self) -> Ty {
        let id = InferId(self.next_infer);
        self.next_infer += 1;
        Ty::Infer(id)
    }

    pub fn spell(&self, t: &Ty) -> String {
        let t = self.resolve(t);
        TySpelling { ty: &t, structs: &self.items.structs, enums: &self.items.enums }.to_string()
    }

    /// Apply the substitution throughout.
    pub fn resolve(&self, t: &Ty) -> Ty {
        match t {
            Ty::Infer(id) => match self.subst.get(id) {
                Some(bound) => self.resolve(bound),
                None => t.clone(),
            },
            Ty::Seq { elem, kind } => Ty::Seq { elem: Box::new(self.resolve(elem)), kind: *kind },
            Ty::Channel(e) => Ty::Channel(Box::new(self.resolve(e))),
            Ty::Ptr { mutable, elem } => Ty::Ptr { mutable: *mutable, elem: Box::new(self.resolve(elem)) },
            Ty::Map { key, val, order } => Ty::Map { key: Box::new(self.resolve(key)), val: Box::new(self.resolve(val)), order: *order },
            Ty::Struct { id, args } => Ty::Struct { id: *id, args: args.iter().map(|a| self.resolve(a)).collect() },
            Ty::Enum { id, args } => Ty::Enum { id: *id, args: args.iter().map(|a| self.resolve(a)).collect() },
            Ty::Fn(f) => Ty::Fn(Box::new(FnTy {
                params: f.params.iter().map(|(k, t)| (*k, self.resolve(t))).collect(),
                ret: self.resolve(&f.ret),
                fails: f.fails.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|t| self.resolve(t)))).collect(),
            })),
            Ty::Bundle(ts) => Ty::Bundle(ts.iter().map(|t| self.resolve(t)).collect()),
            Ty::Bounded { inner, bound } => Ty::Bounded { inner: Box::new(self.resolve(inner)), bound: bound.clone() },
            t => t.clone(),
        }
    }

    /// Make `a` and `b` the same type where one is not yet known. Numeric
    /// widths do not have to agree (a store between widths is a check
    /// site); families and everything else must.
    pub fn unify(&mut self, a: &Ty, b: &Ty) -> bool {
        let a = self.resolve(a);
        let b = self.resolve(b);
        match (&a, &b) {
            (Ty::Infer(x), Ty::Infer(y)) if x == y => true,
            (Ty::Infer(x), other) | (other, Ty::Infer(x)) => {
                self.subst.insert(*x, other.clone());
                true
            }
            (Ty::Int(_), Ty::Int(_)) => true,
            (Ty::Bin(_), Ty::Bin(_)) => true,
            (Ty::Deci(x), Ty::Deci(y)) => x == y,
            (Ty::Bool, Ty::Bool) | (Ty::Duration, Ty::Duration) | (Ty::Nothing, Ty::Nothing) => true,
            (Ty::Str(x), Ty::Str(y)) => matches!((x, y), (Enc::Free(_), _) | (_, Enc::Free(_))) || x == y,
            (Ty::Char(x), Ty::Char(y)) => matches!((x, y), (Unit::Free(_), _) | (_, Unit::Free(_))) || x == y,
            (Ty::Seq { elem: e1, kind: k1 }, Ty::Seq { elem: e2, kind: k2 }) => {
                let kinds = matches!((k1, k2), (SeqKind::Free(_), _) | (_, SeqKind::Free(_))) || k1 == k2;
                kinds && self.unify(e1, e2)
            }
            (Ty::Map { key: k1, val: v1, .. }, Ty::Map { key: k2, val: v2, .. }) => self.unify(k1, k2) && self.unify(v1, v2),
            (Ty::Channel(x), Ty::Channel(y)) => self.unify(x, y),
            (Ty::Ptr { mutable: m1, elem: e1 }, Ty::Ptr { mutable: m2, elem: e2 }) => (m1 == m2 || !*m1) && self.unify(e1, e2),
            (Ty::Struct { id: i1, args: a1 }, Ty::Struct { id: i2, args: a2 }) => i1 == i2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| self.unify(x, y)),
            (Ty::Enum { id: i1, args: a1 }, Ty::Enum { id: i2, args: a2 }) => i1 == i2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(x, y)| self.unify(x, y)),
            (Ty::Fn(f1), Ty::Fn(f2)) => {
                f1.params.len() == f2.params.len()
                    && f1.params.iter().zip(&f2.params).all(|((k1, t1), (k2, t2))| k1 == k2 && self.unify(t1, t2))
                    && self.unify(&f1.ret, &f2.ret)
                    && f1.fails.len() == f2.fails.len()
                    && f1.fails.iter().zip(&f2.fails).all(|((n1, _), (n2, _))| n1 == n2)
            }
            (Ty::Bundle(x), Ty::Bundle(y)) => x.len() == y.len() && x.iter().zip(y).all(|(a, b)| self.unify(a, b)),
            (Ty::Bounded { inner, .. }, other) | (other, Ty::Bounded { inner, .. }) => {
                let inner = inner.clone();
                let other = other.clone();
                self.unify(&inner, &other)
            }
            (Ty::Var(x), Ty::Var(y)) => x == y,
            _ => false,
        }
    }

    /// Strict sameness after resolution (for instance keys).
    pub fn same(&self, a: &Ty, b: &Ty) -> bool {
        self.resolve(a) == self.resolve(b)
    }

    // ------------------------------------------------------------ instances

    /// The instance of `fn_id` with the given shape; queued for checking if new.
    pub fn instance(&mut self, key: InstKey) -> FnInst {
        if let Some(i) = self.inst_index.get(&key) {
            return *i;
        }
        let decl = &self.items.fns[key.fn_id.0 as usize];
        let sig = self.signature_for(decl, &key);
        let inst = FnInst(self.instances.len() as u32);
        self.instances.push(Instance { key: key.clone(), sig, body: None, captures: Vec::new() });
        self.inst_index.insert(key, inst);
        self.pending.push(inst);
        inst
    }

    /// The signature of an instance: blanks substituted, the bundle typed.
    fn signature_for(&self, decl: &crate::resolve::FnDeclInfo, key: &InstKey) -> hir::Signature {
        let mut params = Vec::new();
        let mut blank_iter = 0usize;
        let n_named_blanks = decl.blanks.len();
        for p in &decl.params {
            if p.variadic {
                let bundle = key.bundle.clone().unwrap_or_default();
                params.push((p.kind, self.bundle_type(&bundle)));
                continue;
            }
            let ty = match &p.ty {
                Some(t) => t.subst(&key.blanks),
                None => {
                    // an implicit blank: numbered after the named ones
                    let t = key.blanks.get(n_named_blanks + blank_iter).cloned().unwrap_or(Ty::Nothing);
                    blank_iter += 1;
                    t
                }
            };
            params.push((p.kind, ty));
        }
        let ret = decl.ret.subst(&key.blanks);
        let fails = decl.fails.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|t| t.subst(&key.blanks)))).collect();
        hir::Signature { params, ret, fails }
    }

    /// A variadic bundle's type: all one type → a bounded list of it;
    /// otherwise a tuple, walked by unrolling.
    pub fn bundle_type(&self, tys: &[Ty]) -> Ty {
        if let Some(first) = tys.first() {
            if tys.iter().all(|t| self.same(t, first)) {
                let n = tys.len() as u64;
                return Ty::Bounded { inner: Box::new(Ty::Seq { elem: Box::new(first.clone()), kind: SeqKind::List }), bound: Bound::Len { lo: 0, hi: n, unit: None } };
            }
            Ty::Bundle(tys.to_vec())
        } else {
            Ty::Bundle(Vec::new())
        }
    }

    pub fn check_program(mut self) -> Result<Program> {
        // Claims first: `Vec satisfies Ordered;` is checked at load.
        let claims = self.items.claims.clone();
        for (ty, iface, span) in &claims {
            if let Err(reason) = self.satisfies(ty, *iface) {
                let iname = self.items.ifaces[iface.0 as usize].name.clone();
                return err(Diagnostic::error(format!("the claim `{} satisfies {}` is false: {}", self.spell(ty), iname, reason)).at(*span));
            }
        }
        // Consts, in order.
        for i in 0..self.items.consts.len() {
            let c = self.items.consts[i].clone();
            let mut ctx = self.new_ctx(FnInst(u32::MAX), c.module, "the const".to_string(), Ty::Nothing, Vec::new(), false);
            let mut value = self.expr(&mut ctx, &c.value)?;
            if let Some(t) = &c.ty {
                value = self.expect_store(&mut ctx, value, t, "the const's declared type")?;
            } else {
                let vt = self.resolve(&value.ty);
                self.items.consts[i].ty = Some(vt);
            }
            self.check_const_value(&value)?;
            self.const_values[i] = Some(value);
        }
        // MAIN.
        let main = match self.items.main.clone() {
            Some((block, module, span)) => {
                let inst = FnInst(self.instances.len() as u32);
                let key = InstKey { fn_id: FnId(u32::MAX), blanks: Vec::new(), bundle: None };
                self.instances.push(Instance { key, sig: hir::Signature { params: Vec::new(), ret: Ty::Nothing, fails: Vec::new() }, body: None, captures: Vec::new() });
                let mut ctx = self.new_ctx(inst, module, "MAIN".to_string(), Ty::Nothing, Vec::new(), true);
                let body = self.block(&mut ctx, &block)?;
                self.instances[inst.0 as usize].body = Some(hir::Body { locals: ctx.locals, params: Vec::new(), ret: Ty::Nothing, fails: Vec::new(), block: body, span, name: "MAIN".into(), is_main: true });
                Some(inst)
            }
            None => None,
        };
        // Every instance reached from MAIN (and the consts), transitively.
        while let Some(inst) = self.pending.pop() {
            self.check_instance(inst)?;
        }
        // Functions never called are still checked once, in the abstract
        // shape their declaration gives, so an unused function cannot hide
        // an error. (Generic ones wait for a use: their blanks have no type.)
        for i in 0..self.items.fns.len() {
            let f = &self.items.fns[i];
            if f.is_literal || !f.blanks.is_empty() || f.params.iter().any(|p| p.variadic || p.ty.is_none()) {
                continue;
            }
            let key = InstKey { fn_id: FnId(i as u32), blanks: Vec::new(), bundle: None };
            if !self.inst_index.contains_key(&key) {
                self.instance(key);
                while let Some(inst) = self.pending.pop() {
                    self.check_instance(inst)?;
                }
            }
        }
        let const_values = self.const_values.into_iter().map(|v| v.unwrap()).collect();
        Ok(Program { items: self.items, instances: self.instances, main, const_values, notes: self.notes, amp_count: self.amp_count })
    }

    fn check_const_value(&self, e: &hir::Expr) -> Result<()> {
        // A const is a value known at compile time: literals, arithmetic, other consts, pure calls.
        match &e.kind {
            hir::ExprKind::Int(_) | hir::ExprKind::Bin(_) | hir::ExprKind::Str(_) | hir::ExprKind::Char(_) | hir::ExprKind::Bool(_) | hir::ExprKind::Duration(_) | hir::ExprKind::Const(_) | hir::ExprKind::ListEmpty | hir::ExprKind::MapEmpty | hir::ExprKind::SizeOf { .. } => Ok(()),
            hir::ExprKind::Unary { expr, .. } => self.check_const_value(expr),
            hir::ExprKind::Binary { lhs, rhs, .. } => {
                self.check_const_value(lhs)?;
                self.check_const_value(rhs)
            }
            hir::ExprKind::List(items) => {
                for a in items {
                    if let hir::Arg::Value(v) = a {
                        self.check_const_value(v)?;
                    }
                }
                Ok(())
            }
            hir::ExprKind::Call { callee: hir::Callee::Fn(_), args, .. } => {
                for a in args {
                    if let hir::Arg::Value(v) = a {
                        self.check_const_value(v)?;
                    }
                }
                Ok(())
            }
            hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::Std(name)), .. } => err(Diagnostic::error(format!("a const is a value known at compile time: `std::{}` has an effect, so it cannot make one", name)).at(e.span)),
            hir::ExprKind::Call { callee: hir::Callee::Extern(_), .. } => err(Diagnostic::error("a const is a value known at compile time: a call into C cannot make one").at(e.span)),
            _ => err(Diagnostic::error("a const is a value known at compile time: a literal, arithmetic on literals, another const, or a pure call").at(e.span)),
        }
    }

    pub fn new_ctx(&self, inst: FnInst, module: ModuleId, name: String, ret: Ty, fails: Vec<(String, Option<Ty>)>, is_main: bool) -> FnCtx {
        FnCtx { inst, module, locals: Vec::new(), scopes: vec![HashMap::new()], ret, fails, loop_depth: 0, is_main, is_task: false, name, captured_outer: None, tier: None, blank_names: Vec::new(), literal_lengths: HashMap::new() }
    }

    fn check_instance(&mut self, inst: FnInst) -> Result<()> {
        let key = self.instances[inst.0 as usize].key.clone();
        let sig = self.instances[inst.0 as usize].sig.clone();
        let decl = self.items.fns[key.fn_id.0 as usize].clone();
        let Some(body) = &decl.body else { return Ok(()) };
        let mut ctx = self.new_ctx(inst, decl.module, decl.name.clone(), sig.ret.clone(), sig.fails.clone(), false);
        ctx.blank_names = decl.blanks.iter().map(|(n, _)| n.clone()).collect();
        // Captures of a literal come first, then the parameters.
        let captures = self.instances[inst.0 as usize].captures.clone();
        for (name, verb, ty) in &captures {
            let kind = match verb {
                ast::Verb::Lend => LocalKind::Lent,
                ast::Verb::LendMut => LocalKind::LentMut,
                _ => LocalKind::Owned,
            };
            ctx.declare(name, Local { name: name.clone(), ty: ty.clone(), mutability: None, kind, from_blank: false, span: decl.span });
        }
        let mut params = Vec::new();
        for (p, (kind, ty)) in decl.params.iter().zip(&sig.params) {
            if ctx.lookup(&p.name).is_some() {
                return err(Diagnostic::error(format!("'{}' is declared twice: a capture and a parameter cannot share a name", p.name)).at(p.span));
            }
            let lk = match kind {
                ParamKind::Owned => LocalKind::Owned,
                ParamKind::Lent => LocalKind::Lent,
                ParamKind::LentMut => LocalKind::LentMut,
            };
            if p.amp {
                self.amp_count += 1;
            }
            let id = ctx.declare(&p.name, Local { name: p.name.clone(), ty: ty.clone(), mutability: p.mutability, kind: lk, from_blank: p.from_blank, span: p.span });
            params.push(id);
        }
        let block = self.block(&mut ctx, body)?;
        if sig.ret != Ty::Nothing && !stmt::leaves_block(&block) {
            return err(Diagnostic::error(format!("`{}` returns a `{}` but can fall off the end and return nothing: every path must `return` a value (or `fail`)", decl.name, self.spell(&sig.ret))).at(decl.name_span).why(format!("declared `-> {}`", self.spell(&sig.ret))));
        }
        let name = decl.name.clone();
        self.instances[inst.0 as usize].body = Some(hir::Body { locals: ctx.locals, params, ret: sig.ret, fails: sig.fails, block, span: decl.span, name, is_main: false });
        Ok(())
    }

    /// Register an anonymous function literal as a synthetic declaration.
    pub fn declare_literal(&mut self, lit: &ast::FnLit, module: ModuleId, span: Span) -> Result<FnId> {
        let mut blanks = Vec::new();
        let mut params = Vec::new();
        for p in &lit.params {
            let ty = match &p.ty {
                Some(t) => Some(self.items.resolve_type(t, module, &mut blanks, true)?),
                None => None,
            };
            params.push(crate::resolve::ParamDecl { kind: crate::resolve::conv_kind(p.kind), amp: p.amp, mutability: p.mutability, variadic: p.variadic, name: p.name.name.clone(), ty, from_blank: false, span: p.span });
        }
        let ret = match &lit.ret {
            Some(t) => self.items.resolve_type(t, module, &mut blanks, true)?,
            None => Ty::Nothing,
        };
        let fails = self.items.resolve_fails(&lit.fails, module, &mut blanks)?;
        if !blanks.is_empty() {
            return err(Diagnostic::error("an anonymous function has no blanks: write the types").at(span));
        }
        let id = FnId(self.items.fns.len() as u32);
        self.items.fns.push(crate::resolve::FnDeclInfo { name: "an anonymous function".into(), path: String::new(), module, vis: ast::Vis::Private, params, ret, fails, blanks: Vec::new(), body: Some(lit.body.clone()), op: None, is_literal: true, span, name_span: span });
        self.literal_fns.push((lit.clone(), module));
        Ok(id)
    }

    // ------------------------------------------------------------ interfaces

    /// Does `ty` keep the promise? `Err` carries the reason in plain words.
    pub fn satisfies(&mut self, ty: &Ty, iface: IfaceId) -> std::result::Result<(), String> {
        let ty = self.resolve(ty);
        let info = self.items.ifaces[iface.0 as usize].clone();
        for e in &info.entries {
            if e.is_op {
                if !self.has_op(&ty, &e.name) {
                    return Err(format!("no `op {}` for it", e.name));
                }
            } else if !self.has_fn(&ty, &e.name) {
                return Err(format!("no `fn {}` for it", e.name));
            }
        }
        Ok(())
    }

    /// The language's operators a type keeps, plus user `op`s on it.
    pub fn has_op(&mut self, ty: &Ty, op: &str) -> bool {
        let base = self.resolve(ty);
        let base = base.base().clone();
        let builtin = match &base {
            Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) => matches!(op, "+" | "-" | "x" | "xx" | "/" | "mod" | "==" | "!==" | "<" | ">" | "<==" | ">=="),
            Ty::Bool => matches!(op, "==" | "!=="),
            Ty::Str(_) | Ty::Char(_) => matches!(op, "==" | "!=="),
            Ty::Duration => matches!(op, "==" | "!==" | "<" | ">" | "<==" | ">==" | "+" | "-"),
            Ty::Enum { .. } => op == "==" || op == "!==",
            _ => false,
        };
        if builtin {
            return true;
        }
        self.find_op(op, &base).is_some()
    }

    /// A user `op` by spelling whose left operand is `left`.
    pub fn find_op(&mut self, op: &str, left: &Ty) -> Option<FnId> {
        let candidates = self.items.ops.get(op).cloned().unwrap_or_default();
        for id in candidates {
            let decl = &self.items.fns[id.0 as usize];
            if let Some(Some(t)) = decl.params.first().map(|p| p.ty.clone()) {
                if self.same(&t, left) || (t.contains_var() && !left.contains_var()) {
                    return Some(id);
                }
            }
        }
        None
    }

    pub fn has_fn(&mut self, ty: &Ty, name: &str) -> bool {
        for (i, decl) in self.items.fns.iter().enumerate() {
            let _ = i;
            if decl.name == name && decl.op.is_none() {
                if let Some(Some(t)) = decl.params.first().map(|p| p.ty.clone()) {
                    if self.same(&t, ty) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn root_module(&self) -> ModuleId {
        ROOT
    }

    pub fn iface_name(&self, i: IfaceId) -> &str {
        &self.items.ifaces[i.0 as usize].name
    }

    pub fn struct_name(&self, i: StructId) -> &str {
        &self.items.structs[i.0 as usize].name
    }

    pub fn enum_name(&self, i: EnumId) -> &str {
        &self.items.enums[i.0 as usize].name
    }

    pub fn tier_of(ctx: &FnCtx) -> Option<Tier> {
        ctx.tier
    }

    pub fn mutability_word(m: Option<Mutability>) -> &'static str {
        match m {
            Some(Mutability::Immut) => "immut",
            Some(Mutability::Mut) => "mut",
            None => "",
        }
    }
}
