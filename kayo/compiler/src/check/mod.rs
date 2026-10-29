//! The checker: from the parsed Kit to `hir` — every name resolved,
//! every expression typed, every call bound, every verb agreed with its
//! parameter, every failure handled. Ownership *flow* (a value given
//! twice, used after `give`, a lend outliving its owner) is MIR's job;
//! the checker settles what each spelling *means*.
//!
//! Functions are checked per **instance**: one source function, one
//! shape (generic blanks filled from the call, a variadic bundle typed by
//! its arguments) — the per-shape compilation `generics.md` decides.

pub mod effects;
pub mod expr;
pub mod msl;
pub mod mutscan;
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
    /// The parameters handed a function that carries what it captured
    /// (`copy`/`give`): in this instance they own it, and give it back
    /// (owner, 2026-09-24: a — `run[give 'count']`).
    pub owning: Vec<usize>,
    /// The plain list parameters handed a narrowed list, and its kind
    /// (owner, 2026-10-25): this instance is built knowing it.
    pub pages: Vec<(usize, bool)>,
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
    /// What each inference variable turned out to be. MIR reads types
    /// through it, so nothing downstream sees a type that is still a
    /// question.
    pub subst: HashMap<InferId, Ty>,
    pub main: Option<FnInst>,
    /// Consts, evaluated in order before MAIN.
    pub const_values: Vec<hir::Expr>,
    pub notes: Vec<Diagnostic>,
}

/// One use of a power, as the checker meets it (`cca/LIMBS.md` §4).
#[derive(Debug, Clone)]
pub struct PowerUse {
    pub module: crate::resolve::ModuleId,
    pub path: Vec<String>,
    pub thing: Option<String>,
    /// "calls C (`write`)", for the message.
    pub doing: String,
    pub span: crate::source::Span,
}

pub struct Checker {
    /// Every use of a power met while checking (owner, 2026-10-07).
    pub power_uses: Vec<PowerUse>,
    /// The type a value is stored into, and where the value is: what a
    /// `C.str.make` written there makes (owner, 2026-10-19: a bounded
    /// `C.str` it may not fit).
    pub store_hint: Option<(Ty, Span)>,
    /// The type a parameter or a typed declaration asks of the value about
    /// to be checked, taken by `expr_in` before anything inside it: a bare
    /// enum value there names its type's case (owner, 2026-10-20).
    pub want: Option<Ty>,
    /// Checking a value whose place is a plain number (a typed `let`, a
    /// parameter): a `%` literal in it is the number over 100, not a
    /// percent (owner, 2026-10-17: `let 'f': f64 = 80% x 2.0;` is 1.6).
    pub plain_numbers: bool,
    /// The call or value being checked was written with `std::`, now
    /// taken off to reach the built-in: not a bare std name.
    pub via_std: bool,
    pub items: Items,
    pub instances: Vec<Instance>,
    inst_index: HashMap<InstKey, FnInst>,
    /// Instances whose bodies still wait to be checked.
    pending: Vec<FnInst>,
    /// Instances being checked right now: a body checked early to learn
    /// what its returned borrow depends on must not be entered twice.
    checking: std::collections::HashSet<FnInst>,
    /// Inference variables: what each `Infer` has been unified with.
    pub subst: HashMap<InferId, Ty>,
    next_infer: u32,
    /// Each `mut` variable, by where it is declared, and whether anything
    /// changed it (owner, 2026-10-10).
    pub mut_seen: HashMap<crate::source::Span, (String, bool)>,
    pub const_values: Vec<Option<hir::Expr>>,
    pub notes: Vec<Diagnostic>,
    /// Anonymous `fn` literals get a synthetic FnId each.
    literal_fns: Vec<(ast::FnLit, ModuleId)>,
    /// A testing aid only: type `std::` calls as the oracle emulates them, so the
    /// checker can be run over the examples before std exists. Off by default;
    /// nothing is generated for them.
    pub std_stubs: bool,
    /// A shift, `not.bit` or wrapping operator whose operands are all bare
    /// literals takes its width from where the value goes (owner,
    /// 2026-09-23). Each one's width is a variable a typed place binds;
    /// any still unbound when checking ends had no context at all.
    pub widths_pending: Vec<(Ty, crate::source::Span, String)>,
}

/// The state while checking one body.
pub struct FnCtx {
    /// Loop variables that are a list's lent elements held as plain
    /// numbers: a change to one is a change through a lend, not to a counter.
    pub lent_elements: std::collections::HashSet<LocalId>,
    pub inst: FnInst,
    pub module: ModuleId,
    pub locals: Vec<Local>,
    /// Scopes: name → local, innermost last.
    pub scopes: Vec<HashMap<String, LocalId>>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
    pub loop_depth: u32,
    pub is_main: bool,
    /// In a `test fn`: `must` is allowed (owner, 2026-09-29).
    pub is_test: bool,
    /// Checking `must.fail`'s call: it needs no `otherwise`.
    pub must_fail: bool,
    pub is_task: bool,
    pub name: String,
    /// Names captured by an enclosing fn literal are visible only through `with`.
    pub captured_outer: Option<Vec<(String, ast::Verb, Ty, LocalId)>>,
    pub tier: Option<Tier>,
    /// A generic instance: the blanks that were filled, by name, for messages.
    pub blank_names: Vec<String>,
    /// Locals holding a list literal whose length is still known (for `...`).
    pub literal_lengths: HashMap<LocalId, usize>,
    /// Locals declared `!$`: they may not be shadowed (owner, 2026-09-25).
    pub sealed: std::collections::HashSet<LocalId>,
    /// A local declared with a literal and never changeable, and the
    /// literal as it was typed: what `print.written` prints (owner,
    /// 2026-10-17).
    pub written: HashMap<LocalId, String>,
    /// `-> lent@x T` (owner, 2026-09-29): whether it may change, its group.
    pub ret_lent: Option<(bool, Option<String>)>,
    /// The `lent` parameters: their position and group.
    pub lent_params: HashMap<LocalId, (usize, Option<String>)>,
    /// The parameters a `return lend …` handed back, by position.
    pub returned: std::collections::BTreeSet<usize>,
    /// The one call whose returned borrow a declaration is taking now.
    pub lend_call_ok: Option<crate::source::Span>,
    /// What each borrowing local borrows, followed back to owners or
    /// `lent` parameters: a borrow of one is a borrow of those.
    pub borrow_src: HashMap<LocalId, Vec<LocalId>>,
    /// Its variables are `case.any` (owner, 2026-10-06): held and
    /// found by their folded names.
    pub fold_vars: bool,
    /// The locals declared `mut`, a promise that they change (owner,
    /// 2026-10-10: "`b`"), and the locals something changes.
    pub promised: Vec<LocalId>,
    pub changed: std::collections::HashSet<LocalId>,
    /// The consts declared in a body (owner, 2026-10-10: "`a`"), scope by
    /// scope beside `scopes`.
    pub const_scopes: Vec<HashMap<String, crate::hir::ConstId>>,
    /// Checking a body const's value: the variables around it, which it may
    /// not use.
    pub outer_vars: Option<Vec<String>>,
    /// Checking a const's value (owner, 2026-10-10: `incl.bytes` is one
    /// only).
    pub in_const: bool,
    /// Each const array read in place here, and the local that views it,
    /// set up as the body starts.
    pub const_views: Vec<(crate::hir::ConstId, LocalId)>,
    /// What a `:=` whole number can be, from its operands (owner,
    /// 2026-10-12: "(a)"), for whether it fits a `deci`; its first range.
    pub int_ranges: HashMap<LocalId, (i128, i128)>,
}

impl FnCtx {
    fn key<'a>(&self, name: &'a str) -> std::borrow::Cow<'a, str> {
        if self.fold_vars { std::borrow::Cow::Owned(crate::ast::fold_case(name)) } else { std::borrow::Cow::Borrowed(name) }
    }
    /// The name in scope the one written most likely meant, a few letters
    /// off: a variable or a const.
    pub fn close_name(&self, name: &str) -> Option<String> {
        let names = self.scopes.iter().flat_map(|s| s.keys()).chain(self.const_scopes.iter().flat_map(|s| s.keys()));
        crate::diag::closest(name, names.map(|k| k.as_str())).map(str::to_string)
    }

    /// Refused as not declared, with the close name when there is one.
    pub fn not_declared(&self, d: crate::diag::Diagnostic, name: &str, at: crate::source::Span) -> Box<crate::diag::Diagnostic> {
        let twin = self.scopes.iter().flat_map(|s| s.keys()).find(|k| k.as_str() != name && k.to_lowercase() == name.to_lowercase()).cloned();
        Box::new(match twin.or_else(|| self.close_name(name)) {
            // Every name is exact (owner, 2026-10-27): one differing only in case is asked about.
            Some(c) if c.to_lowercase() == name.to_lowercase() => {
                let mut d = d;
                d.sentence = format!("`${}` is not declared; did you mean `${}`? Every name is exact (owner, 2026-10-27)", name, c);
                d.fix_edit(at, crate::diag::Edit::Replace, crate::diag::name_written(&c))
            }
            Some(c) => d.fix_edit(at, crate::diag::Edit::Replace, crate::diag::name_written(&c)),
            None => d,
        })
    }

    pub fn lookup(&self, name: &str) -> Option<LocalId> {
        let name = self.key(name);
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.get(name.as_ref()) {
                return Some(*id);
            }
        }
        None
    }
    pub fn declare(&mut self, name: &str, local: Local) -> LocalId {
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(local);
        let name = self.key(name).into_owned();
        self.scopes.last_mut().unwrap().insert(name, id);
        id
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.const_scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        self.const_scopes.pop();
    }
    /// A const declared in this body and seen here.
    pub fn lookup_const(&self, name: &str) -> Option<crate::hir::ConstId> {
        let name = self.key(name);
        self.const_scopes.iter().rev().find_map(|s| s.get(name.as_ref()).copied())
    }
    /// Is `name` declared in an *enclosing* scope (not the innermost)?
    pub fn visible_outside(&self, name: &str) -> Option<LocalId> {
        let name = self.key(name);
        for scope in self.scopes.iter().rev().skip(1) {
            if let Some(id) = scope.get(name.as_ref()) {
                return Some(*id);
            }
        }
        None
    }
    pub fn in_current_scope(&self, name: &str) -> Option<LocalId> {
        self.scopes.last().unwrap().get(self.key(name).as_ref()).copied()
    }
}

impl Checker {
    pub fn new(items: Items) -> Checker {
        let n = items.consts.len();
        Checker { power_uses: Vec::new(), store_hint: None, want: None, plain_numbers: false, via_std: false, items, instances: Vec::new(), inst_index: HashMap::new(), pending: Vec::new(), subst: HashMap::new(), next_infer: 1, const_values: vec![None; n], notes: Vec::new(), literal_fns: Vec::new(), checking: Default::default(), std_stubs: false, widths_pending: Vec::new(), mut_seen: HashMap::new() }
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

    /// A type as the writer writes it in module `m`, for a fix: std's own
    /// types take `std::` outside std (owner, 2026-10-22), as `spell`,
    /// which is for sentences, leaves out.
    pub fn spell_written(&self, m: crate::resolve::ModuleId, t: &Ty) -> String {
        let s = self.spell(t);
        // Inside std, or in a kit that does not list it, they are bare.
        let root = self.items.kit_root(m);
        if self.items.nerve_cord.is_some_and(|c| c == root) || (self.items.nerve_cord.is_none() && self.items.no_std_roots.contains(&root)) {
            return s;
        }
        let mut out = String::with_capacity(s.len() + 8);
        let mut rest = s.as_str();
        while !rest.is_empty() {
            let before = out.chars().last();
            let joined = before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ':');
            let head = ["list", "map", "duration"].into_iter().find(|h| rest.starts_with(h) && !rest[h.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_'));
            match head {
                Some(h) if !joined => {
                    out.push_str("std::");
                    out.push_str(h);
                    rest = &rest[h.len()..];
                }
                _ => {
                    let c = rest.chars().next().unwrap();
                    out.push(c);
                    rest = &rest[c.len_utf8()..];
                }
            }
        }
        out
    }

    /// Apply the substitution throughout.
    pub fn resolve(&self, t: &Ty) -> Ty {
        match t {
            Ty::Infer(id) => match self.subst.get(id) {
                Some(bound) => self.resolve(bound),
                None => t.clone(),
            },
            // A whole number not pinned yet, which a later use may pin.
            Ty::Int(crate::types::IntW::Free(id)) if id.0 != 0 => match self.subst.get(id) {
                Some(bound) => self.resolve(bound),
                None => t.clone(),
            },
            Ty::Seq { elem, kind } => Ty::Seq { elem: Box::new(self.resolve(elem)), kind: *kind },
            Ty::Channel(e) => Ty::Channel(Box::new(self.resolve(e))),
            // A lane count of 0 means "the shape this target has for that
            // lane type": there is only ever one (`simd.md`, owner: b), so
            // the element type decides it and nothing has to be inferred.
            Ty::Simd { elem, lanes } => {
                let e = self.resolve(elem);
                let n = if *lanes == 0 { crate::types::simd_shapes_for(&e).unwrap_or(0) } else { *lanes };
                Ty::Simd { elem: Box::new(e), lanes: n }
            }
            Ty::Ptr { mutable, elem } => Ty::Ptr { mutable: *mutable, elem: Box::new(self.resolve(elem)) },
            Ty::Map { key, val, order } => Ty::Map { key: Box::new(self.resolve(key)), val: Box::new(self.resolve(val)), order: *order },
            Ty::Struct { id, args } => Ty::Struct { id: *id, args: args.iter().map(|a| self.resolve(a)).collect() },
            Ty::Enum { id, args } => Ty::Enum { id: *id, args: args.iter().map(|a| self.resolve(a)).collect() },
            Ty::Fn(f) => Ty::Fn(Box::new(FnTy {
                params: f.params.iter().map(|(k, t)| (*k, self.resolve(t))).collect(),
                ret: self.resolve(&f.ret),
                fails: f.fails.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|t| self.resolve(t)))).collect(),
                owning: f.owning,
                proc: f.proc,
            })),
            Ty::Bundle(ts) => Ty::Bundle(ts.iter().map(|t| self.resolve(t)).collect()),
            Ty::Bounded { inner, bound } => Ty::Bounded { inner: Box::new(self.resolve(inner)), bound: bound.clone() },
            Ty::Percent(inner) => Ty::Percent(Box::new(self.resolve(inner))),
            Ty::Places { inner, max, min } => Ty::Places { inner: Box::new(self.resolve(inner)), max: *max, min: *min },
            Ty::Pages { inner, aligned } => Ty::Pages { inner: Box::new(self.resolve(inner)), aligned: *aligned },
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
            // An unknown type first known to be a bare whole number is one
            // whose width a later use may still pin: a `list.empty` given
            // bare numbers, then handed back as a `list[i32]`, is a list
            // of `i32`, laid out as one (examples/bounded: it was read as
            // a list of 64-bit numbers).
            (Ty::Infer(x), Ty::Int(crate::types::IntW::Free(InferId(0)))) | (Ty::Int(crate::types::IntW::Free(InferId(0))), Ty::Infer(x)) => {
                let id = InferId(self.next_infer);
                self.next_infer += 1;
                self.subst.insert(*x, Ty::Int(crate::types::IntW::Free(id)));
                true
            }
            (Ty::Infer(x), other) | (other, Ty::Infer(x)) => {
                self.subst.insert(*x, other.clone());
                true
            }
            (Ty::Int(crate::types::IntW::Free(x)), Ty::Int(crate::types::IntW::Free(y))) if x.0 != 0 && y.0 != 0 && x != y => {
                self.subst.insert(*x, b.clone());
                true
            }
            (Ty::Int(crate::types::IntW::Free(x)), fixed @ Ty::Int(crate::types::IntW::Fixed { .. })) | (fixed @ Ty::Int(crate::types::IntW::Fixed { .. }), Ty::Int(crate::types::IntW::Free(x))) if x.0 != 0 => {
                self.subst.insert(*x, fixed.clone());
                true
            }
            (Ty::Int(_), Ty::Int(_)) => true,
            (Ty::Bin(_), Ty::Bin(_)) => true,
            (Ty::Deci(x), Ty::Deci(y)) => x == y,
            (Ty::Bool, Ty::Bool) | (Ty::Duration, Ty::Duration) | (Ty::Writer, Ty::Writer) | (Ty::Nothing, Ty::Nothing) | (Ty::Brain, Ty::Brain) => true,
            (Ty::Str(x), Ty::Str(y)) => matches!((x, y), (Enc::Free(_), _) | (_, Enc::Free(_))) || x == y,
            (Ty::Char(x), Ty::Char(y)) => matches!((x, y), (Unit::Free(_), _) | (_, Unit::Free(_))) || x == y,
            (Ty::Seq { elem: e1, kind: k1 }, Ty::Seq { elem: e2, kind: k2 }) => {
                let kinds = matches!((k1, k2), (SeqKind::Free(_), _) | (_, SeqKind::Free(_))) || k1 == k2;
                kinds && self.unify(e1, e2)
            }
            (Ty::Map { key: k1, val: v1, .. }, Ty::Map { key: k2, val: v2, .. }) => self.unify(k1, k2) && self.unify(v1, v2),
            (Ty::Channel(x), Ty::Channel(y)) => self.unify(x, y),
            (Ty::Simd { elem: e1, lanes: l1 }, Ty::Simd { elem: e2, lanes: l2 }) => (l1 == l2 || *l1 == 0 || *l2 == 0) && self.unify(e1, e2),
            // A pointer whose kind nothing has said yet, `ptr.null` (owner,
            // 2026-10-21), fits a `ptr` and a `ptr.mut` alike.
            (Ty::Ptr { mutable: m1, elem: e1 }, Ty::Ptr { mutable: m2, elem: e2 }) => {
                let open = matches!(self.resolve(e1), Ty::Infer(_)) || matches!(self.resolve(e2), Ty::Infer(_));
                (m1 == m2 || !*m1 || open) && self.unify(e1, e2)
            }
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
            // A percent is its own family (owner, 2026-10-17): the same as
            // another percent of the same number, nothing else.
            (Ty::Percent(x), Ty::Percent(y)) => {
                let (x, y) = (x.clone(), y.clone());
                self.unify(&x, &y)
            }
            // A list narrowed to its memory meets only the same narrowing
            // (design claude, 2026-10-25: none crosses into another until
            // the owner rules).
            (Ty::Pages { inner: x, aligned: a }, Ty::Pages { inner: y, aligned: b }) => {
                let (x, y) = (x.clone(), y.clone());
                a == b && self.unify(&x, &y)
            }
            (Ty::Pages { .. }, _) | (_, Ty::Pages { .. }) => false,
            (Ty::Places { inner, .. }, other) | (other, Ty::Places { inner, .. }) => {
                let inner = inner.clone();
                let other = other.clone();
                self.unify(&inner, &other)
            }
            (Ty::Bounded { inner, .. }, other) | (other, Ty::Bounded { inner, .. }) => {
                let inner = inner.clone();
                let other = other.clone();
                self.unify(&inner, &other)
            }
            (Ty::Var(x), Ty::Var(y)) => x == y,
            _ => false,
        }
    }

    /// A narrowed list (`std::list.page_aligned[T]` or `.not_page_aligned`)
    /// going where a plain `std::list[T]` is taken: its kind, `true` for
    /// page-aligned (owner, 2026-10-25). The other way round needs
    /// `to.list.page_aligned(…)`.
    pub fn narrowed_into(&mut self, value: &Ty, target: &Ty) -> Option<bool> {
        let (value, target) = (self.resolve(value), self.resolve(target));
        let Ty::Pages { inner, aligned } = &value else { return None };
        if crate::precog::paged::narrowed(&target).is_some() || !matches!(target.base(), Ty::Seq { kind: SeqKind::List | SeqKind::Free(_), .. }) {
            return None;
        }
        let inner = (**inner).clone();
        self.unify(&inner, &target).then_some(*aligned)
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
            let ty = match ty {
                Ty::Fn(mut f) if key.owning.contains(&params.len()) => {
                    f.owning = true;
                    Ty::Fn(f)
                }
                t => match key.pages.iter().find(|(i, _)| *i == params.len()) {
                    Some((_, aligned)) => Ty::Pages { inner: Box::new(t), aligned: *aligned },
                    None => t,
                },
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
        self.check_op_meanings()?;
        // A program is its `MAIN { … }` block; `fn MAIN` is an ordinary
        // function (owner, 2026-10-26), so a root with it and no block has
        // nothing to run.
        if self.items.main.is_none() {
            if let Some(f) = self.items.fns.iter().find(|f| f.name == "MAIN" && f.module == crate::resolve::ROOT) {
                return err(Diagnostic::error("this kit has no `MAIN { … }`, the program's block; `fn MAIN` is an ordinary function, called `my.MAIN()`").named("main_form").at(f.name_span));
            }
        }
        // Claims first: `Vec satisfies Ordered;` is checked at load.
        let claims = self.items.claims.clone();
        for (ty, iface, span) in &claims {
            if let Err(reason) = self.satisfies(ty, *iface) {
                let iname = self.items.ifaces[iface.0 as usize].name.clone();
                return err(Diagnostic::error(format!("the claim `{} satisfies {}` is false: {}", self.spell(ty), iname, reason)).named("false_satisfies_claim").at(*span));
            }
        }
        // Consts, in order.
        for i in 0..self.items.consts.len() {
            let c = self.items.consts[i].clone();
            let mut ctx = self.new_ctx(FnInst(u32::MAX), c.module, "the const".to_string(), Ty::Nothing, Vec::new(), false);
            ctx.in_const = true;
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
            Some((block, module, span, main_params)) => {
                let inst = FnInst(self.instances.len() as u32);
                let key = InstKey { fn_id: FnId(u32::MAX), blanks: Vec::new(), bundle: None, owning: Vec::new(), pages: Vec::new() };
                self.instances.push(Instance { key, sig: hir::Signature { params: Vec::new(), ret: Ty::Nothing, fails: Vec::new() }, body: None, captures: Vec::new() });
                let mut ctx = self.new_ctx(inst, module, "MAIN".to_string(), Ty::Nothing, Vec::new(), true);
                ctx.fold_vars = self.items.case_vars.contains(&span);
                // C's `argc` and `argv` (owner, 2026-09-29): exactly these
                // two types, the names the writer's.
                let mut params = Vec::new();
                let want = [("argc", Ty::Int(crate::types::IntW::Fixed { signed: true, bits: 32 })), ("argv", Ty::Ptr { mutable: false, elem: Box::new(Ty::Ptr { mutable: false, elem: Box::new(Ty::Int(crate::types::IntW::Fixed { signed: false, bits: 8 })) }) })];
                for (p, (c, want)) in main_params.iter().zip(want.iter()) {
                    let Some(t) = &p.ty else {
                        return err(Diagnostic::error(format!("MAIN's '{}' is C's `{}`: write its type, `{}`", p.name.name, c, self.spell(want))).named("main_parameters_wrong").at(p.span));
                    };
                    let mut blanks = Vec::new();
                    let got = self.items.resolve_type(t, module, &mut blanks, false)?;
                    if got != *want || !matches!(p.kind, ast::ParamKind::Owned) {
                        return err(Diagnostic::error(format!("MAIN's '{}' is C's `{}`, whose type is `{}`, not `{}`", p.name.name, c, self.spell(want), self.spell(&got))).named("main_parameters_wrong").at(t.span));
                    }
                    let id = ctx.declare(&p.name.name, Local { name: p.name.name.clone(), ty: want.clone(), mutability: None, kind: LocalKind::Owned, from_blank: false, span: p.span });
                    params.push(id);
                }
                let mut body = self.block(&mut ctx, &block)?;
                self.mut_promises_kept(&ctx)?;
                self.const_views_first(&ctx, &mut body);
                self.instances[inst.0 as usize].body = Some(hir::Body { locals: ctx.locals, params, ret: Ty::Nothing, fails: Vec::new(), block: body, span, name: "MAIN".into(), is_main: true, is_test: false, ret_lent: None });
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
            let key = InstKey { fn_id: FnId(i as u32), blanks: Vec::new(), bundle: None, owning: Vec::new(), pages: Vec::new() };
            if !self.inst_index.contains_key(&key) {
                self.instance(key);
                while let Some(inst) = self.pending.pop() {
                    self.check_instance(inst)?;
                }
            }
        }
        // A function no instance checked — a generic nobody calls — has its
        // `mut`s read from its source (owner, 2026-10-10: "`a`").
        let checked: std::collections::HashSet<u32> = self.instances.iter().map(|i| i.key.fn_id.0).collect();
        for i in 0..self.items.fns.len() {
            let f = &self.items.fns[i];
            if checked.contains(&(i as u32)) || f.is_literal {
                continue;
            }
            let Some(body) = &f.body else { continue };
            let mut scan = mutscan::Scan::default();
            for p in &f.params {
                scan.declare(&p.name, p.span, p.mutability == Some(Mutability::Mut));
            }
            scan.block(body);
            // An operator on a `<T>` value comes from a `sig` in its bound that lists it (owner,
            // 2026-10-28: (b)): read from the source here too, its blanks having no type.
            for (spelled, at, names) in &scan.ops {
                for n in names {
                    let Some(p) = f.params.iter().find(|p| &p.name == n) else { continue };
                    let Some(Ty::Var(b)) = &p.ty else { continue };
                    let Some((blank, bounds)) = f.blanks.get(*b as usize) else { continue };
                    let sets = bounds.iter().any(|i| !self.items.ifaces[i.0 as usize].types.is_empty());
                    let listed = bounds.iter().any(|i| self.items.ifaces[i.0 as usize].entries.iter().any(|e| e.is_op && &e.name == spelled));
                    if sets && !listed {
                        let shown = crate::surface::op_spoken(spelled);
                        return err(Diagnostic::error(format!("`{}` on a `<{}>` comes from a `sig` in its bound listing `op {}`: a `set` gives a body only its types (owner, 2026-10-28)", shown, blank, shown)).named("unknown_operator").at(*at));
                    }
                }
            }
            for v in scan.vars.into_iter().filter(|v| v.promised) {
                self.mut_seen.entry(v.span).or_insert((v.name, v.changed));
            }
        }
        // A `mut` nothing changed, in any instance.
        let mut unkept: Vec<(&crate::source::Span, &String)> = self.mut_seen.iter().filter(|(_, (_, ok))| !ok).map(|(s, (n, _))| (s, n)).collect();
        unkept.sort_by_key(|(s, _)| (s.file.0, s.start));
        if let Some((span, name)) = unkept.first() {
            return err(Diagnostic::error(format!("'{}' is a `var`, but nothing changes it: make it a `let`, or change it (`var` is a promise that it changes, as `let` is that it does not; owner, 2026-10-27)", name)).named("mut_never_changed").at(**span));
        }
        // "Only a bare literal with no context has no width" (`operators.md`).
        for (t, span, what) in std::mem::take(&mut self.widths_pending) {
            let got = self.resolve(&t);
            // Bound, but to something that is not a whole number: an integer
            // operation stored into a `bool` is a mistake, as `'a': bool
            // = 5` is.
            if !matches!(got, Ty::Infer(_)) && !got.is_int() {
                return err(Diagnostic::error(format!("expected a value of the declared type ({}), found a whole number: `{}` works on integers", self.spell(&got), what)).named("needs_integer").at(span));
            }
            if matches!(got, Ty::Infer(_)) {
                return err(Diagnostic::error(format!("`{}` needs a width, and nothing here gives one: every operand is a bare number and the value is not stored into a typed place", what)).named("width_unknown")
                    .at(span)
                    .try_(format!("give the result a type (`let $x: u32 = …`) or one operand a width (`to.u32(…)`)")));
            }
        }
        self.check_powers()?;
        // A call's brackets written wrong, once nothing else is (owner,
        // 2026-10-22: calls take `( )`).
        if let Some(d) = crate::parser::first_wrong_bracket() {
            return Err(Box::new(d));
        }
        let const_values = self.const_values.into_iter().map(|v| v.unwrap()).collect();
        Ok(Program { items: self.items, instances: self.instances, subst: self.subst, main, const_values, notes: self.notes })
    }

    fn check_const_value(&self, e: &hir::Expr) -> Result<()> {
        // A const is a value known at compile time: literals, arithmetic, other consts, pure calls.
        match &e.kind {
            hir::ExprKind::Int(_) | hir::ExprKind::Bin(_) | hir::ExprKind::Deci { .. } | hir::ExprKind::Str(_) | hir::ExprKind::Char(_) | hir::ExprKind::Bool(_) | hir::ExprKind::Duration(_) | hir::ExprKind::Const(_) | hir::ExprKind::ListEmpty | hir::ExprKind::MapEmpty | hir::ExprKind::SizeOf { .. } => Ok(()),
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
            // A file's bytes, read while building (owner, 2026-10-10).
            hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::InclBytes), .. } => Ok(()),
            hir::ExprKind::Call { callee: hir::Callee::Fn(_), args, .. } => {
                for a in args {
                    if let hir::Arg::Value(v) = a {
                        self.check_const_value(v)?;
                    }
                }
                Ok(())
            }
            hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::Std(name)), .. } => err(Diagnostic::error(format!("a const is a value known at compile time: `std::{}` has an effect, so it cannot make one", name)).named("const_not_known").at(e.span)),
            hir::ExprKind::Call { callee: hir::Callee::Extern(_), .. } => err(Diagnostic::error("a const is a value known at compile time: a call into C cannot make one").named("const_not_known").at(e.span)),
            _ => err(Diagnostic::error("a const is a value known at compile time: a literal, arithmetic on literals, another const, or a pure call").named("const_not_known").at(e.span)),
        }
    }

    /// `mut` promises that the variable changes, as `immut` that it does
    /// not (owner, 2026-10-10: "`b`"). Each body says which of its `mut`
    /// variables something changed — a `change`, a `lend.mut` (a task's or
    /// a function literal's capture too), an `inline.ASM` output — by the
    /// variable, so a shadow is another; and since a function is checked
    /// once per shape (a variadic with no values never walks them), the
    /// promise is kept if any of its instances changes it.
    /// The const arrays this body reads in place, each viewed by its local
    /// from the start (owner, 2026-10-10).
    pub fn const_views_first(&self, ctx: &FnCtx, block: &mut hir::Block) {
        let mut first = Vec::new();
        for (cid, id) in &ctx.const_views {
            let l = &ctx.locals[id.0 as usize];
            let (ty, span) = (l.ty.clone(), l.span);
            let c = hir::Expr { kind: hir::ExprKind::Const(*cid), ty: ty.clone(), span };
            let view = hir::Expr { kind: hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::ConstView), args: vec![hir::Arg::Value(c)], otherwise: None, fails: Vec::new() }, ty, span };
            first.push(hir::Stmt { kind: hir::StmtKind::Let { local: *id, value: view, typed: false }, span, tier: None });
        }
        block.stmts.splice(0..0, first);
    }

    pub fn mut_promises_kept(&mut self, ctx: &FnCtx) -> crate::diag::Result<()> {
        for id in &ctx.promised {
            let l = &ctx.locals[id.0 as usize];
            let e = self.mut_seen.entry(l.span).or_insert((l.name.clone(), false));
            e.1 |= ctx.changed.contains(id);
        }
        Ok(())
    }

    /// A type written inside a body (`let 'sum': <T> = 0;`): the function's
    /// blanks are its own, filled with this instance's types.
    pub(crate) fn body_type(&self, ctx: &FnCtx, t: &ast::Type) -> Result<Ty> {
        let mut blanks: Vec<(String, Vec<crate::types::IfaceId>)> = ctx.blank_names.iter().map(|n| (n.clone(), Vec::new())).collect();
        let generic = !ctx.blank_names.is_empty();
        let rt = self.items.resolve_type(t, ctx.module, &mut blanks, generic)?;
        Ok(match self.instances.get(ctx.inst.0 as usize) {
            Some(i) if generic => rt.subst(&i.key.blanks),
            _ => rt,
        })
    }

    pub fn new_ctx(&self, inst: FnInst, module: ModuleId, name: String, ret: Ty, fails: Vec<(String, Option<Ty>)>, is_main: bool) -> FnCtx {
                    FnCtx { lent_elements: Default::default(), inst, module, locals: Vec::new(), scopes: vec![HashMap::new()], ret, fails, loop_depth: 0, is_main, is_test: false, must_fail: false, is_task: false, name, captured_outer: None, tier: None, blank_names: Vec::new(), literal_lengths: HashMap::new(), sealed: Default::default(), written: HashMap::new(), ret_lent: None, lent_params: HashMap::new(), returned: Default::default(), lend_call_ok: None, borrow_src: HashMap::new(), fold_vars: false, promised: Vec::new(), changed: Default::default(), const_scopes: vec![HashMap::new()], outer_vars: None, in_const: false, const_views: Vec::new(), int_ranges: HashMap::new() }
    }

    fn check_instance(&mut self, inst: FnInst) -> Result<()> {
        self.checking.insert(inst);
        let r = self.check_instance_body(inst);
        self.checking.remove(&inst);
        r
    }

    /// Check a callee now, if it waits, so what its returned borrow depends
    /// on is known: the parameters its body hands back (owner, 2026-09-29).
    pub fn returned_deps(&mut self, inst: FnInst) -> Result<Option<Vec<usize>>> {
        if self.instances[inst.0 as usize].body.is_none() && !self.checking.contains(&inst) {
            self.pending.retain(|i| *i != inst);
            self.check_instance(inst)?;
        }
        Ok(self.instances[inst.0 as usize].body.as_ref().and_then(|b| b.ret_lent.as_ref().map(|(_, d)| d.clone())))
    }

    /// Why a `fn` (or a `coll.fn`'s member, or a `fn` lambda) is a `proc`'s body, as the oracle
    /// says it.
    pub fn not_a_fn(&self, decl: &crate::resolve::FnDeclInfo, f: effects::Found) -> Diagnostic {
        let (sentence, name, at) = match f {
            effects::Found::Effect(what, at) if decl.is_literal => (format!("a `fn [ … ]` lambda has no effects, but this one {}: write it `proc [ … ] = …` (owner, 2026-10-27)", what), "fn_has_effect", at),
            effects::Found::Effect(what, at) if decl.coll_member => (format!("the member `{}` is in a `coll.fn`, which has no effects, but it {}: make the collection a `coll.proc` (owner, 2026-10-27)", decl.name, what), "fn_has_effect", at),
            effects::Found::Effect(what, at) => (format!("`{}` is a `fn`, which has no effects, but it {}: make it a `proc` (owner, 2026-10-27)", decl.name, what), "fn_has_effect", at),
            effects::Found::Proc(callee, at) if decl.is_literal => (format!("a `fn [ … ]` lambda calls only `fn`s, but this one calls `{}`, a `proc`: write it `proc [ … ] = …` (owner, 2026-10-27)", callee), "fn_calls_proc", at),
            effects::Found::Proc(callee, at) if decl.coll_member => (format!("the member `{}` is in a `coll.fn`, which calls only `fn`s, but `{}` is a `proc`: make the collection a `coll.proc` (owner, 2026-10-27)", decl.name, callee), "fn_calls_proc", at),
            effects::Found::Proc(callee, at) => (format!("`{}` is a `fn`, so it calls only `fn`s, but `{}` is a `proc`: make `{}` a `proc`, or `{}` a `fn` (owner, 2026-10-27)", decl.name, callee, decl.name, callee), "fn_calls_proc", at),
            effects::Found::ProcValue(held, at) if decl.is_literal => (format!("a `fn [ … ]` lambda calls only `fn`s, but {} holds a `proc[…]`: write it `proc [ … ] = …` (owner, 2026-10-27)", held), "fn_calls_proc", at),
            effects::Found::ProcValue(held, at) => (format!("`{}` is a `fn`, so it calls only `fn`s, but {} holds a `proc[…]`: make `{}` a `proc`, or {} a `fn[…]` (owner, 2026-10-27)", decl.name, held, decl.name, held), "fn_calls_proc", at),
        };
        let mut d = Diagnostic::error(sentence).named(name).at(at);
        if !decl.is_literal {
            d = d.why_at(decl.name_span, format!("`{}` is declared a `fn` here", decl.name));
        }
        d
    }

    fn check_instance_body(&mut self, inst: FnInst) -> Result<()> {
        let key = self.instances[inst.0 as usize].key.clone();
        let sig = self.instances[inst.0 as usize].sig.clone();
        let decl = self.items.fns[key.fn_id.0 as usize].clone();
        let Some(body) = &decl.body else { return Ok(()) };
        let mut ctx = self.new_ctx(inst, decl.module, decl.name.clone(), sig.ret.clone(), sig.fails.clone(), false);
        ctx.fold_vars = self.items.case_vars.contains(&decl.name_span);
        ctx.is_test = decl.test;
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
        ctx.ret_lent = decl.ret_lent.clone();
        for (pi, (p, (kind, ty))) in decl.params.iter().zip(&sig.params).enumerate() {
            if ctx.lookup(&p.name).is_some() {
                return err(Diagnostic::error(format!("'{}' is declared twice: a capture and a parameter cannot share a name", p.name)).named("already_declared").at(p.span));
            }
            let lk = match kind {
                ParamKind::Owned => LocalKind::Owned,
                ParamKind::Lent => LocalKind::Lent,
                ParamKind::LentMut => LocalKind::LentMut,
            };
            let id = ctx.declare(&p.name, Local { name: p.name.clone(), ty: ty.clone(), mutability: p.mutability, kind: lk, from_blank: p.from_blank, span: p.span });
            if p.mutability == Some(Mutability::Mut) {
                ctx.promised.push(id);
            }
            if matches!(kind, ParamKind::Lent | ParamKind::LentMut) {
                ctx.lent_params.insert(id, (pi, p.group.clone()));
            }
            params.push(id);
        }
        let block = self.block(&mut ctx, body)?;
        // A `fn` has no effects and calls only `fn`s (owner, 2026-10-27).
        if !decl.proc && !decl.test && decl.op.is_none() {
            let mut w = effects::Walk { checker: self, locals: &ctx.locals, found: None };
            w.block(&block);
            if let Some(f) = w.found {
                return err(self.not_a_fn(&decl, f));
            }
        }
        // What the caller's result borrows: the group's parameters, or,
        // with no group written, the ones the body hands back (Precog's
        // inference, owner 2026-09-29).
        let ret_lent = decl.ret_lent.as_ref().map(|(m, g)| {
            let deps: Vec<usize> = match g {
                Some(g) => decl.params.iter().enumerate().filter(|(_, p)| p.group.as_ref() == Some(g)).map(|(i, _)| i).collect(),
                None => ctx.returned.iter().copied().collect(),
            };
            (*m, deps)
        });
        if sig.ret != Ty::Nothing && !stmt::leaves_block(&block) {
            return err(Diagnostic::error(format!("`{}` returns a `{}` but can fall off the end and return nothing: every path must `return` a value (or `fail`)", decl.name, self.spell(&sig.ret))).named("missing_return").at(decl.name_span).why(format!("declared `-> {}`", self.spell(&sig.ret))));
        }
        let name = decl.name.clone();
        self.mut_promises_kept(&ctx)?;
        let mut block = block;
        self.const_views_first(&ctx, &mut block);
        self.instances[inst.0 as usize].body = Some(hir::Body { locals: ctx.locals, params, ret: sig.ret, fails: sig.fails, block, span: decl.span, name, is_main: false, is_test: decl.test, ret_lent });
        Ok(())
    }

    /// Register an anonymous function literal as a synthetic declaration.
    pub fn declare_literal(&mut self, lit: &ast::FnLit, module: ModuleId, span: Span, from: Option<&FnTy>) -> Result<FnId> {
        let mut blanks = Vec::new();
        let mut params = Vec::new();
        for (i, p) in lit.params.iter().enumerate() {
            // A short lambda's parameter: written, kept; left out, where it
            // goes says it (owner, 2026-10-24).
            let ty = match (&p.ty, from) {
                (Some(t), _) => Some(self.items.resolve_type(t, module, &mut blanks, true)?),
                (None, Some(ft)) => Some(ft.params[i].1.clone()),
                (None, None) => None,
            };
            params.push(crate::resolve::ParamDecl { kind: crate::resolve::conv_kind(p.kind), group: None, amp: p.amp, mutability: p.mutability, variadic: p.variadic, name: p.name.name.clone(), ty, from_blank: false, span: p.span });
        }
        let ret = match (&lit.ret, from) {
            (Some(t), _) => self.items.resolve_type(t, module, &mut blanks, true)?,
            (None, Some(ft)) => ft.ret.clone(),
            (None, None) => Ty::Nothing,
        };
        let fails = match from {
            Some(ft) => ft.fails.clone(),
            None => self.items.resolve_fails(&lit.fails, module, &mut blanks)?,
        };
        if !blanks.is_empty() {
            return err(Diagnostic::error("an anonymous function has no blanks: write the types").named("parameter_type_missing").at(span));
        }
        let id = FnId(self.items.fns.len() as u32);
        self.items.fns.push(crate::resolve::FnDeclInfo { name: "an anonymous function".into(), path: String::new(), module, vis: ast::Vis::Private, params, ret, ret_lent: None, fails, blanks: Vec::new(), body: Some(lit.body.clone()), op: None, is_literal: true, test: false, proc: lit.proc, coll_member: false, table_bytes: None, labels: None, defaults: Vec::new(), span, name_span: span });
        self.literal_fns.push((lit.clone(), module));
        Ok(id)
    }

    // ------------------------------------------------------------ interfaces

    /// Does `ty` keep the promise? `Err` carries the reason in plain words.
    pub fn satisfies(&mut self, ty: &Ty, iface: IfaceId) -> std::result::Result<(), String> {
        let ty = self.resolve(ty);
        let info = self.items.ifaces[iface.0 as usize].clone();
        // A type list is closed: the type is one of them. A listed integer
        // admits every integer, as the width is Precog's to pick.
        if !info.types.is_empty() {
            let same_family = |listed: &Ty| match (listed.base(), ty.base()) {
                (Ty::Int(_), Ty::Int(_)) | (Ty::Bin(_), Ty::Bin(_)) | (Ty::Deci(_), Ty::Deci(_)) => true,
                (a, b) => a == b,
            };
            if !info.types.iter().any(same_family) {
                let listed: Vec<String> = info.types.iter().map(|t| self.spell(t)).collect();
                return Err(format!("it is not one of the types it lists ({})", listed.join(", ")));
            }
        }
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
            Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) => matches!(op, "plus" | "minus" | "times" | "pow" | "equal_to" | "not_equal" | "<" | ">" | "at_most" | "at_least") || op.starts_with("div.") || op.starts_with("mod."),
            Ty::Bool => matches!(op, "equal_to" | "not_equal"),
            Ty::Str(_) | Ty::Char(_) => matches!(op, "equal_to" | "not_equal"),
            Ty::Duration => matches!(op, "equal_to" | "not_equal" | "<" | ">" | "at_most" | "at_least" | "plus" | "minus"),
            Ty::Enum { .. } => op == "equal_to" || op == "not_equal",
            _ => false,
        };
        if builtin {
            return true;
        }
        self.find_op(op, &base).is_some()
    }

    /// One meaning per operator and types (owner, 2026-09-29): two `op`s
    /// of one name for the same types, a generic one covering a specific
    /// one's type (the specific does not win), and two generic ones of one
    /// name are each an error; ops of one name for different types are
    /// fine. A generic op never takes a language operator's name.
    fn check_op_meanings(&mut self) -> Result<()> {
        let mut names: Vec<(String, Vec<FnId>)> = self.items.ops.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        names.sort_by(|a, b| a.0.cmp(&b.0));
        for (op, ids) in names {
            let decls: Vec<(FnId, crate::resolve::FnDeclInfo)> = ids.iter().map(|id| (*id, self.items.fns[id.0 as usize].clone())).collect();
            let language = matches!(op.as_str(), "plus" | "minus" | "times" | "pow" | "equal_to" | "not_equal" | "<" | ">" | "at_most" | "at_least" | "++" | "xor" | "shl" | "shr" | "plus.wrap" | "minus.wrap" | "times.wrap" | "pow.wrap") || op.starts_with("div") || op.starts_with("mod");
            for (_, d) in &decls {
                if language && !d.blanks.is_empty() {
                    return err(Diagnostic::error(format!("a generic operator cannot be `{}`: that is the language's own, and a generic one would take it over for every type", op)).named("builtin_operator_redefined").at(d.name_span));
                }
            }
            for (i, (_, a)) in decls.iter().enumerate() {
                for (_, b) in decls.iter().skip(i + 1) {
                    // Two Kits may each give a word operator its meaning for
                    // the same types: the path tells them apart (owner,
                    // 2026-10-02). A glyph operator has none.
                    if op.starts_with(|c: char| c.is_ascii_alphabetic()) && self.items.kit_root(a.module) != self.items.kit_root(b.module) {
                        continue;
                    }
                    let (ag, bg) = (!a.blanks.is_empty(), !b.blanks.is_empty());
                    let types = |d: &crate::resolve::FnDeclInfo| -> Vec<Ty> { d.params.iter().filter_map(|p| p.ty.clone()).collect() };
                    match (ag, bg) {
                        (false, false) => {
                            let (ta, tb) = (types(a), types(b));
                            if ta.len() == tb.len() && ta.iter().zip(&tb).all(|(x, y)| self.same(x, y)) {
                                let spelled = ta.iter().map(|t| self.spell(t)).collect::<Vec<_>>().join(", ");
                                return err(Diagnostic::error(format!("`{}` is defined twice for {}: one meaning per operator and types", op, spelled)).named("versions_overlap").at(b.name_span).why_at(a.name_span, "the first is here"));
                            }
                        }
                        (true, true) => {
                            return err(Diagnostic::error(format!("`{}` is declared generic twice: one generic meaning per operator", op)).named("versions_overlap").at(b.name_span).why_at(a.name_span, "the first is here"));
                        }
                        _ => {
                            let (g, s) = if ag { (a, b) } else { (b, a) };
                            // The specific op's left operand, held to the
                            // generic's first blank's interface.
                            let Some(Some(st)) = s.params.first().map(|p| p.ty.clone()) else { continue };
                            let covers = match g.params.first().and_then(|p| p.ty.clone()) {
                                Some(Ty::Var(v)) => match g.blanks.get(v as usize).map(|(_, b)| b.clone()) {
                                    Some(bounds) => bounds.iter().all(|i| self.satisfies(&st, *i).is_ok()),
                                    None => true,
                                },
                                _ => false,
                            };
                            if covers {
                                let spelled = self.spell(&st);
                                return err(Diagnostic::error(format!("`{}` for {} and the generic `{}` both cover {}: one meaning per operator and types, and the specific one does not win", op, spelled, op, spelled)).named("versions_overlap").at(s.name_span).why_at(g.name_span, "the generic one is here"));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// A user `op` by spelling whose left operand is `left`.
    pub fn find_op(&mut self, op: &str, left: &Ty) -> Option<FnId> {
        self.find_op_in(op, left, None)
    }

    /// The same, only among the operators of the Kit whose root is `kit`
    /// when one is named: `'x' areas::mean 'y'` (owner, 2026-10-03).
    pub fn find_op_in(&mut self, op: &str, left: &Ty, kit: Option<ModuleId>) -> Option<FnId> {
        self.ops_matching(op, left, kit, true).into_iter().next()
    }

    /// Every operator `op` whose left operand takes `left`, in the Kit
    /// whose root is `kit` when one is named; the first only when `first`.
    pub fn ops_matching(&mut self, op: &str, left: &Ty, kit: Option<ModuleId>, first: bool) -> Vec<FnId> {
        let mut found = Vec::new();
        let mut candidates = self.items.ops.get(op).cloned().unwrap_or_default();
        if let Some(root) = kit {
            candidates.retain(|id| self.items.kit_root(self.items.fns[id.0 as usize].module) == root);
        }
        for id in candidates {
            let decl = &self.items.fns[id.0 as usize];
            if let Some(Some(t)) = decl.params.first().map(|p| p.ty.clone()) {
                // A bare number on the left takes the operand's type, as it
                // does beside a language operator: `20 gap 3`; an exact
                // answer lands in it, as in any call.
                let bare = match (left.base(), t.base()) {
                    (Ty::Int(crate::types::IntW::Free(_)), Ty::Int(_)) => true,
                    (Ty::Int(_), Ty::Int(_)) if left.is_exact() => true,
                    (Ty::Bin(crate::types::BinW::Free(_)), Ty::Bin(_) | Ty::Brain) => true,
                    _ => false,
                };
                if bare || self.same(&t, left) || (t.contains_var() && !left.contains_var()) {
                    found.push(id);
                    if first {
                        return found;
                    }
                }
            }
        }
        found
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

impl Checker {
    /// Note a use of a power, for the ledger.
    pub(crate) fn power_use(&mut self, ctx: &FnCtx, path: &[&str], thing: Option<String>, doing: String, span: crate::source::Span) {
        self.power_uses.push(PowerUse { module: ctx.module, path: path.iter().map(|p| p.to_string()).collect(), thing, doing, span });
    }

    /// The power ledger (`cca/LIMBS.md` step 2; owner, 2026-10-07): each
    /// use, in source order, granted, listed, not denied. A Kit with no
    /// `CCA.cplr` is not checked: it has every core and power.
    fn check_powers(&mut self) -> Result<()> {
        // An operator on built-in types is a use where it is declared:
        // `operators_on_builtin_types` when every operand is built in,
        // `prefix_operators` with one operand (owner: "`a`").
        let builtin = |t: &Option<crate::types::Ty>| t.as_ref().is_some_and(|t| matches!(t.base(), crate::types::Ty::Int(_) | crate::types::Ty::Bin(_) | crate::types::Ty::Brain | crate::types::Ty::Deci(_) | crate::types::Ty::Bool | crate::types::Ty::Str(_) | crate::types::Ty::Char(_)));
        let mut uses = std::mem::take(&mut self.power_uses);
        for f in &self.items.fns {
            let Some(op) = &f.op else { continue };
            if f.params.is_empty() || !f.params.iter().all(|p| builtin(&p.ty)) {
                continue;
            }
            let power = if f.params.len() == 1 { "prefix_operators" } else { "operators_on_builtin_types" };
            uses.push(PowerUse { module: f.module, path: vec![power.to_string()], thing: None, doing: format!("defines the operator `{}` on built-in types", op), span: f.name_span });
        }
        uses.sort_by_key(|u| (u.span.file.0, u.span.start));
        for u in &uses {
            if let Some((name, sentence)) = self.power_refused(u) {
                return err(Diagnostic::error(sentence).at(u.span).named(name));
            }
        }
        Ok(())
    }

    /// Why a use of a power is refused, if it is, with the error's name.
    fn power_refused(&self, u: &PowerUse) -> Option<(&'static str, String)> {
        use kayo_base::powers::{check, Grant, Kit, Power, Refusal, Use};
        let root = self.items.kit_root(u.module);
        let lp = self.items.kit_powers.get(&root)?;
        // The Brainstem grants these to every Neuron (vouching and the
        // operator powers since IEG joined it, owner 2026-10-12); a Neuron
        // still lists each it uses.
        let grants: Vec<Grant> = ["inline_assembly", "c_calls", "raw_memory", "vouch", "operators_on_builtin_types", "prefix_operators"].iter().map(|p| Grant { power: Power::new(p), by: "the base".into(), to: None }).collect();
        let kit = Kit { name: lp.prefix.clone(), id: None, lists: lp.lists.clone(), grants };
        let used = Use { kit: lp.prefix.clone(), path: u.path.clone(), thing: u.thing.clone(), doing: u.doing.clone(), at: u.span.start };
        let who = if lp.is_program { "this kit".to_string() } else { format!("`{}`", lp.prefix) };
        let top = &u.path[0];
        Some(match check(&used, &kit, &self.items.program_deny) {
            Ok(()) => return None,
            Err(Refusal::NotGranted { .. }) => ("power_not_granted", format!("{} {}, a power nothing grants it: `{}`", who, u.doing, top)),
            Err(Refusal::NotListed { .. }) => ("power_not_listed", format!("{} {}, and its CyJet.toml does not list `{}`: add `{} {{` / `}}` under its `my powers:`", who, u.doing, top, top)),
            Err(Refusal::Denied { entry, .. }) => ("power_denied", format!("{} {}, which this program denies (`my denied powers:`: `{}`)", who, u.doing, entry.power.path.join("."))),
        })
    }

    /// Before refusing an unmarked site: the power it uses, just recorded,
    /// is asked about first, as the oracle asks — a Neuron that may not call
    /// C hears that before how to mark the call.
    pub(crate) fn power_first(&self) -> Result<()> {
        if let Some(u) = self.power_uses.last() {
            if let Some((name, sentence)) = self.power_refused(u) {
                return err(Diagnostic::error(sentence).at(u.span).named(name));
            }
        }
        Ok(())
    }
}

/// Two names a program uses that differ only in case (owner, 2026-10-27): each name is exact, so
/// they are two, and the build report says so. Its functions, types and constants, and each
/// function's variables. Inside std, nothing (it is not the writer's).
pub fn case_twins(program: &Program) -> Vec<(crate::source::Span, String)> {
    let mut out = Vec::new();
    let mut pair = |names: &[(String, crate::source::Span)], what: &str| {
        for (i, (a, at)) in names.iter().enumerate() {
            for (b, _) in &names[..i] {
                if a != b && a.to_lowercase() == b.to_lowercase() {
                    out.push((*at, format!("{} `{}` and `{}` differ only in case: every name is exact, so they are two names (owner, 2026-10-27)", what, b, a)));
                }
            }
        }
    };
    let std_kit = |m: crate::resolve::ModuleId| program.items.nerve_cord.is_some_and(|c| program.items.kit_root(m) == c);
    let mut items: Vec<(String, crate::source::Span)> = Vec::new();
    for f in &program.items.fns {
        if !f.is_literal && f.op.is_none() && !std_kit(f.module) && !items.iter().any(|(n, _)| *n == f.name) {
            items.push((f.name.clone(), f.name_span));
        }
    }
    for s in &program.items.structs {
        if !items.iter().any(|(n, _)| *n == s.name) {
            items.push((s.name.clone(), s.span));
        }
    }
    for e in &program.items.enums {
        if !items.iter().any(|(n, _)| *n == e.name) {
            items.push((e.name.clone(), e.span));
        }
    }
    pair(&items, "the names");
    for inst in &program.instances {
        let Some(body) = &inst.body else { continue };
        // MAIN has no entry among the functions.
        if program.items.fns.get(inst.key.fn_id.0 as usize).is_some_and(|d| std_kit(d.module)) {
            continue;
        }
        let mut vars: Vec<(String, crate::source::Span)> = Vec::new();
        for l in &body.locals {
            if !l.name.contains(':') && !l.name.contains(' ') && !vars.iter().any(|(n, _)| *n == l.name) {
                vars.push((l.name.clone(), l.span));
            }
        }
        let shown: Vec<(String, crate::source::Span)> = vars.into_iter().map(|(n, s)| (format!("${}", n), s)).collect();
        pair(&shown, "the variables");
    }
    out.sort_by_key(|(s, _)| (s.file.0, s.start));
    out.dedup();
    out
}
