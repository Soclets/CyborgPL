//! Attack (c) of the Precog oracle (design/precog.md): a second, separately
//! written translation from a program to solver questions. For every place
//! a generated program can fail — a value landing outside its type, past
//! 64 bits in between, a division by zero, a shift count out of range, a
//! negative exponent — it asks a fresh Z3: "is there an input that reaches
//! here, with nothing failing on the way, and fails here?"
//!
//! The translation follows design/, not either engine: plain integer maths
//! is exact and checked where it lands (owner, dc83c05); `/` floors and
//! `mod` takes the divisor's sign; bitwise and wrapping operators work at
//! their operands' width; a shift count must be 0..width-1.
//!
//! It is used for aiming, never for judging: every input it finds is run
//! through the oracle and both CCA builds, and only a disagreement there is
//! a finding. So an imprecision here (loops cut at a bound, calls too deep
//! left unknown) costs aim, never truth. `and`/`or` run both sides, as the
//! owner ruled (fc04c70).

use crate::ast::*;
use crate::ast::addr;
use std::collections::HashMap;
use std::fmt::Write as _;

/// Bits per value: a value in range is within -2^63 .. 2^64-1 (66 bits
/// signed); sums fit, and a product that would not is caught with Z3's own
/// overflow test before it wraps.
const W: u32 = 72;
/// Inputs are environment-variable lengths the harness can actually set.
pub const INPUT_MAX: u64 = 200_000;
/// Stop translating past this many definitions (a program too big to ask about).
const MAX_DEFS: usize = 60_000;
/// Past this many, calls are no longer inlined: their results become
/// unknown values, so a deep recursion costs aim rather than the program.
const INLINE_DEFS: usize = 20_000;
/// Loop trips unrolled at most (the generator's own loops stay under 13).
const MAX_TRIPS: usize = 14;
/// Recursion inlined at most this deep.
const MAX_DEPTH: usize = 12;

#[derive(Clone, Debug)]
pub struct Site {
    pub what: String,
    /// The kind of check, in `cca precog --json`'s words.
    pub kind: &'static str,
    /// The expression the site belongs to (its address, see ast::Spans),
    /// and for a landing the simple statement it is in (0 if none).
    pub node: usize,
    pub stmt: usize,
    /// For a landing: the type it must fit — base name and range.
    pub fits: Option<(&'static str, i128, i128)>,
}

/// The script, and the sites in the order their `check-sat`s appear.
pub struct Query {
    pub script: String,
    pub sites: Vec<Site>,
    pub inputs: Vec<String>,
}

#[derive(Clone)]
struct SiteRaw {
    reach: String,
    fail: String,
    what: String,
    kind: &'static str,
    node: usize,
    stmt: usize,
    value: String,
    width: u32,
    fits: Option<(&'static str, i128, i128)>,
}

#[derive(Clone)]
struct Var {
    term: String,
    ty: TyR,
}

struct Enc<'a> {
    p: &'a Program,
    defs: String,
    ndefs: usize,
    /// Per site: (reach term, fail term, what, kind, node, stmt, value term, value width)
    sites: Vec<SiteRaw>,
    /// The node being translated when a site is made.
    node: usize,
    stmt: usize,
    /// The call whose arguments are being landed (its span is the site's).
    call_at: usize,
    scopes: Vec<HashMap<String, Var>>,
    too_big: bool,
    depth: usize,
}

fn bv(v: i128) -> String {
    if v >= 0 {
        format!("(_ bv{v} {W})")
    } else {
        format!("(bvneg (_ bv{} {W}))", -v)
    }
}

fn range_of(t: TyR) -> (i128, i128) {
    let r = t.base.range_true();
    match t.range {
        None => (r.0, r.1),
        Some((lo, hi)) => (lo.unwrap_or(r.0), hi.unwrap_or(r.1)),
    }
}

impl Ty {
    /// The type's real range, whatever the generator's own view (NARROW_64).
    fn range_true(self) -> (i128, i128) {
        if self.signed() {
            (-(1i128 << (self.bits() - 1)), (1i128 << (self.bits() - 1)) - 1)
        } else {
            (0, (1i128 << self.bits()) - 1)
        }
    }
}

/// Where a program runs: the path condition, and whether the current
/// block has been left (break / continue / return) on some paths.
#[derive(Clone)]
struct Ctx {
    pc: String,
}

/// Flags for leaving blocks: set under a condition, read as "still here".
#[derive(Default, Clone)]
struct Leave {
    brk: Option<String>,
    cont: Option<String>,
    ret: Option<String>,
    /// For a function: its return value so far (an ite chain).
    ret_val: Option<String>,
    ret_ty: Option<TyR>,
}

impl<'a> Enc<'a> {
    fn def(&mut self, sort: &str, term: String) -> String {
        self.ndefs += 1;
        if self.ndefs > MAX_DEFS {
            self.too_big = true;
        }
        let name = format!("t{}", self.ndefs);
        let _ = writeln!(self.defs, "(define-fun {name} () {sort} {term})");
        name
    }
    fn int(&mut self, term: String) -> String {
        self.def(&format!("(_ BitVec {W})"), term)
    }
    fn boolean(&mut self, term: String) -> String {
        self.def("Bool", term)
    }
    fn fresh_int(&mut self) -> String {
        self.ndefs += 1;
        let name = format!("u{}", self.ndefs);
        let _ = writeln!(self.defs, "(declare-const {name} (_ BitVec {W}))");
        name
    }

    fn site(&mut self, ctx: &Ctx, fail: String, what: String, kind: &'static str, value: &str, width: u32) {
        let f = self.boolean(fail);
        self.sites.push(SiteRaw { reach: ctx.pc.clone(), fail: f, what, kind, node: self.node, stmt: self.stmt, value: value.to_string(), width, fits: None });
    }

    fn in_range(&self, v: &str, lo: i128, hi: i128) -> String {
        format!("(and (bvsle {} {v}) (bvsle {v} {}))", bv(lo), bv(hi))
    }

    /// A value landing in `t`: a site if it can be outside.
    fn land(&mut self, ctx: &Ctx, v: &str, t: TyR, what: &str) {
        if t.base == Ty::Bool {
            return;
        }
        let (lo, hi) = range_of(t);
        let fail = format!("(not {})", self.in_range(v, lo, hi));
        self.site(ctx, fail, format!("{what} into {}", t.text()), "narrowing", v, W);
        self.sites.last_mut().unwrap().fits = Some((t.base.name(), lo, hi));
    }

    /// Plain exact maths: a result no 64-bit type holds is a site.
    fn limit64(&mut self, ctx: &Ctx, v: &str, what: &str) {
        let fail = format!("(not {})", self.in_range(v, -(1i128 << 63), (1i128 << 64) - 1));
        self.site(ctx, fail, format!("{what} past 64 bits"), "past_64_bits", v, W);
    }

    fn lookup(&self, n: &str) -> Option<Var> {
        self.scopes.iter().rev().find_map(|s| s.get(n).cloned())
    }

    fn set(&mut self, n: &str, term: String) {
        for s in self.scopes.iter_mut().rev() {
            if let Some(v) = s.get_mut(n) {
                v.term = term;
                return;
            }
        }
    }

    // ---------------------------------------------------------------- expressions

    /// The type an integer expression works at (for widths of bitwise
    /// operators): the first name, conversion or call inside it.
    fn ty_of(&self, e: &Expr) -> Option<Ty> {
        match e {
            Expr::Var(n) => self.lookup(n).map(|v| v.ty.base),
            Expr::To(t, _) => Some(*t),
            Expr::Call(f, _) => self.p.funcs.iter().find(|g| &g.name == f).map(|g| g.ret.base),
            Expr::Bin(_, a, b) => self.ty_of(a).or_else(|| self.ty_of(b)),
            Expr::Neg(a) | Expr::NotBit(a) => self.ty_of(a),
            _ => None,
        }
    }

    fn expr(&mut self, ctx: &Ctx, e: &Expr, ctx_ty: Option<Ty>) -> String {
        if self.too_big {
            return bv(0);
        }
        // Bare numbers alone: their exact answer, no sites in between.
        if !matches!(e, Expr::Lit(_)) {
            if let Some(v) = fold_bare(e) {
                return bv(v.clamp(-(1i128 << (W - 1)), (1i128 << (W - 1)) - 1));
            }
        }
        match e {
            // A list's items and length are not modelled: any value, which
            // the evaluator's confirmation of every refutation makes safe.
            Expr::At(_, p) => {
                self.expr(ctx, p, Some(Ty::U64));
                self.fresh_int()
            }
            Expr::Len(_) | Expr::Lend(_) => self.fresh_int(),
            // An enum value alone as an integer: only its case (see `enum_val`).
            Expr::Case(..) | Expr::ToEnum(..) => self.enum_val(ctx, e).0,
            // A match giving a value is encoded by its statement (`Let`),
            // which sees arms that leave; anywhere else it is not modelled.
            Expr::Match(..) | Expr::SimdFill(_) | Expr::SimdFold(..) => {
                self.too_big = true;
                self.fresh_int()
            }
            Expr::Lit(v) => bv(*v),
            Expr::Bool(b) => b.to_string(),
            Expr::Var(n) => self.lookup(n).map(|v| v.term).unwrap_or_else(|| self.fresh_int()),
            Expr::Neg(a) => {
                let x = self.expr(ctx, a, ctx_ty);
                self.node = addr(e);
                let r = self.int(format!("(bvneg {x})"));
                self.limit64(ctx, &r, "unary -");
                r
            }
            Expr::Not(a) => {
                let x = self.expr(ctx, a, None);
                self.boolean(format!("(not {x})"))
            }
            Expr::And(a, b) => {
                let (x, y) = (self.expr(ctx, a, None), self.expr(ctx, b, None));
                self.boolean(format!("(and {x} {y})"))
            }
            Expr::Or(a, b) => {
                let (x, y) = (self.expr(ctx, a, None), self.expr(ctx, b, None));
                self.boolean(format!("(or {x} {y})"))
            }
            // The right side runs, and its sites are reached, only where the
            // left side has not decided.
            Expr::AndThen(a, b) | Expr::OrElse(a, b) => {
                let x = self.expr(ctx, a, None);
                let then = matches!(e, Expr::AndThen(..));
                let pc = self.boolean(if then { format!("(and {} {x})", ctx.pc) } else { format!("(and {} (not {x}))", ctx.pc) });
                let y = self.expr(&Ctx { pc }, b, None);
                self.boolean(if then { format!("(and {x} {y})") } else { format!("(or {x} {y})") })
            }
            Expr::Cmp(op, a, b) if self.is_enum(a) || self.is_enum(b) => {
                let (k1, d1) = self.enum_val(ctx, a);
                let (k2, d2) = self.enum_val(ctx, b);
                let same = format!("(and (= {k1} {k2}) (= {d1} {d2}))");
                self.boolean(if *op == CmpOp::Eq { same } else { format!("(not {same})") })
            }
            Expr::Cmp(op, a, b) => {
                let t = self.ty_of(a).or_else(|| self.ty_of(b));
                let (x, y) = (self.expr(ctx, a, t), self.expr(ctx, b, t));
                let f = match op {
                    CmpOp::Eq => format!("(= {x} {y})"),
                    CmpOp::Ne => format!("(not (= {x} {y}))"),
                    CmpOp::Lt => format!("(bvslt {x} {y})"),
                    CmpOp::Gt => format!("(bvsgt {x} {y})"),
                    CmpOp::Le => format!("(bvsle {x} {y})"),
                    CmpOp::Ge => format!("(bvsge {x} {y})"),
                };
                self.boolean(f)
            }
            Expr::To(t, a) => {
                let x = self.expr(ctx, a, Some(*t));
                self.node = addr(e);
                self.stmt = 0;
                self.land(ctx, &x, (*t).into(), &format!("to.{}", t.name()));
                x
            }
            Expr::NotBit(a) => {
                let t = self.ty_of(a).or(ctx_ty).unwrap_or(Ty::I64);
                let x = self.expr(ctx, a, Some(t));
                if t.signed() {
                    self.int(format!("(bvnot {x})"))
                } else {
                    self.int(format!("(bvsub {} {x})", bv(t.range_true().1)))
                }
            }
            Expr::Call(f, args) => {
                // an argument's landing is reported at the call (CCA's span)
                let saved = std::mem::replace(&mut self.call_at, addr(e));
                let r = self.call(ctx, f, args);
                self.call_at = saved;
                r
            }
            Expr::Bin(op, a, b) => {
                let t = self.ty_of(a).or_else(|| self.ty_of(b)).or(ctx_ty).unwrap_or(Ty::I64);
                let x = self.expr(ctx, a, Some(t));
                let y = self.expr(ctx, b, Some(t));
                self.node = addr(e);
                self.binary(ctx, *op, &x, &y, t)
            }
        }
    }

    fn wrap(&mut self, v: &str, t: Ty) -> String {
        let w = t.bits();
        let low = format!("((_ extract {} 0) {v})", w - 1);
        let ext = if t.signed() { "sign_extend" } else { "zero_extend" };
        self.int(format!("((_ {ext} {}) {low})", W - w))
    }

    fn binary(&mut self, ctx: &Ctx, op: BinOp, x: &str, y: &str, t: Ty) -> String {
        use BinOp::*;
        let w = t.bits() as i128;
        match op {
            Add | Sub => {
                let f = if op == Add { "bvadd" } else { "bvsub" };
                let r = self.int(format!("({f} {x} {y})"));
                self.limit64(ctx, &r, op.text());
                r
            }
            Mul => {
                // The true product at double width; past 64 bits there is a site,
                // and within 64 bits the low 72 bits are the value.
                let wide = self.def(&format!("(_ BitVec {})", 2 * W), format!("(bvmul ((_ sign_extend {W}) {x}) ((_ sign_extend {W}) {y}))"));
                let lo = format!("((_ sign_extend {W}) {})", bv(-(1i128 << 63)));
                let hi = format!("((_ sign_extend {W}) {})", bv((1i128 << 64) - 1));
                self.site(ctx, format!("(or (bvslt {wide} {lo}) (bvsgt {wide} {hi}))"), "x past 64 bits".into(), "past_64_bits", &wide, 2 * W);
                self.int(format!("((_ extract {} 0) {wide})", W - 1))
            }
            Div | Mod => {
                let kind = if op == Div { "division_by_zero" } else { "modulo_by_zero" };
                self.site(ctx, format!("(= {y} {})", bv(0)), format!("{} by zero", op.text()), kind, y, W);
                // Guard the maths against the zero case (it cannot happen past the site).
                let d = self.int(format!("(ite (= {y} {}) {} {y})", bv(0), bv(1)));
                let m = self.int(format!("(bvsmod {x} {d})"));
                if op == Mod {
                    return m;
                }
                let q = self.int(format!("(bvsdiv (bvsub {x} {m}) {d})"));
                self.limit64(ctx, &q, "/");
                q
            }
            Pow => {
                self.site(ctx, format!("(bvslt {y} {})", bv(0)), "xx with a negative exponent".into(), "negative_exponent", y, W);
                // Unrolled over exponents 0..16; past them the result is unknown.
                // A power that leaves 72 bits on the way is past 64 bits.
                let mut pows = vec![bv(1)];
                let mut acc = bv(1);
                let mut wrapped = "false".to_string();
                let lo = format!("((_ sign_extend {W}) {})", bv(-(1i128 << 63)));
                let hi = format!("((_ sign_extend {W}) {})", bv((1i128 << 64) - 1));
                for _ in 1..=16 {
                    let wide = self.def(&format!("(_ BitVec {})", 2 * W), format!("(bvmul ((_ sign_extend {W}) {acc}) ((_ sign_extend {W}) {x}))"));
                    wrapped = self.boolean(format!("(or {wrapped} (bvslt {wide} {lo}) (bvsgt {wide} {hi}))"));
                    acc = self.int(format!("((_ extract {} 0) {wide})", W - 1));
                    pows.push(acc.clone());
                }
                self.site(ctx, format!("(and (bvsle {y} {}) {wrapped})", bv(16)), "xx past 64 bits".into(), "past_64_bits", "", 0);
                let mut r = self.fresh_int();
                for (k, pk) in pows.iter().enumerate().rev() {
                    r = self.int(format!("(ite (= {y} {}) {pk} {r})", bv(k as i128)));
                }
                self.limit64(ctx, &r, "xx");
                r
            }
            AndBit => self.int(format!("(bvand {x} {y})")),
            OrBit => self.int(format!("(bvor {x} {y})")),
            Xor => self.int(format!("(bvxor {x} {y})")),
            Shl | Shr => {
                self.site(
                    ctx,
                    format!("(or (bvslt {y} {}) (bvsge {y} {}))", bv(0), bv(w)),
                    format!("{} count outside 0..{}", op.text(), w - 1),
                    "shift_count",
                    y,
                    W,
                );
                let k = self.int(format!("(ite (and (bvsge {y} {}) (bvslt {y} {})) {y} {})", bv(0), bv(w), bv(0)));
                if op == Shl {
                    let r = self.int(format!("(bvshl {x} {k})"));
                    let (lo, hi) = t.range_true();
                    // bits pushed out of 72 are caught by shifting back
                    let lost = format!("(not (= (bvashr {r} {k}) {x}))");
                    self.site(ctx, format!("(or {lost} (not {}))", self.in_range(&r, lo, hi)), format!("shl past {}", t.name()), "shift_width", &r, W);
                    r
                } else {
                    self.int(format!("(bvashr {x} {k})"))
                }
            }
            WAdd | WSub | WMul => {
                let f = match op {
                    WAdd => "bvadd",
                    WSub => "bvsub",
                    _ => "bvmul",
                };
                let r = self.int(format!("({f} {x} {y})"));
                self.wrap(&r, t)
            }
        }
    }

    fn call(&mut self, ctx: &Ctx, f: &str, args: &[Expr]) -> String {
        // The printer: only its argument's landing matters.
        if f == "put_i" || f == "put_u" {
            let t = if f == "put_i" { Ty::I64 } else { Ty::U64 };
            let x = self.expr(ctx, &args[0], Some(t));
            (self.node, self.stmt) = (addr(&args[0]), self.call_at);
            self.land(ctx, &x, t.into(), "argument");
            return bv(0);
        }
        let Some(func) = self.p.funcs.iter().find(|g| g.name == f) else { return self.fresh_int() };
        let mut vals = vec![];
        for (a, prm) in args.iter().zip(&func.params) {
            let x = self.expr(ctx, a, Some(prm.ty.base));
            (self.node, self.stmt) = (addr(a), self.call_at);
            self.land(ctx, &x, prm.ty, "argument");
            vals.push(x);
        }
        if self.depth >= MAX_DEPTH || self.too_big || self.ndefs > INLINE_DEFS {
            let r = self.fresh_int();
            return r;
        }
        self.depth += 1;
        let saved = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        for (prm, v) in func.params.iter().zip(vals) {
            self.scopes[0].insert(prm.name.clone(), Var { term: v, ty: prm.ty });
        }
        let mut leave = Leave { ret: Some("false".into()), ret_val: Some(bv(0)), ret_ty: Some(func.ret), ..Default::default() };
        self.block(ctx, &func.body, &mut leave);
        self.scopes = saved;
        self.depth -= 1;
        leave.ret_val.unwrap()
    }

    // ---------------------------------------------------------------- statements

    // ---------------------------------------------------------------- enums

    fn is_enum(&self, e: &Expr) -> bool {
        match e {
            Expr::Case(..) | Expr::ToEnum(..) => true,
            Expr::Var(n) => self.lookup(&format!("{n}#d")).is_some(),
            _ => false,
        }
    }

    /// An enum value as two terms: its case's index and its data (0 if none).
    fn enum_val(&mut self, ctx: &Ctx, e: &Expr) -> (String, String) {
        match e {
            Expr::Var(n) => {
                let k = self.lookup(n).map(|v| v.term).unwrap_or_else(|| bv(0));
                let d = self.lookup(&format!("{n}#d")).map(|v| v.term).unwrap_or_else(|| bv(0));
                (k, d)
            }
            Expr::ToEnum(en, x, fallback) => {
                // The case whose number it is, else the fallback, whose sites
                // are reached only where no case matched.
                let v = self.expr(ctx, x, Some(Ty::I64));
                let vals = self.p.enums.iter().find(|d| &d.name == en).and_then(|d| d.values.clone()).unwrap_or_default();
                let hit = if vals.is_empty() { "false".to_string() } else { format!("(or {})", vals.iter().map(|w| format!("(= {v} {})", bv(*w))).collect::<Vec<_>>().join(" ")) };
                let miss = self.boolean(format!("(and {} (not {hit}))", ctx.pc));
                let (fk, fd) = self.enum_val(&Ctx { pc: miss }, fallback);
                let mut k = fk;
                for (i, w) in vals.iter().enumerate().rev() {
                    k = self.int(format!("(ite (= {v} {}) {} {k})", bv(*w), bv(i as i128)));
                }
                let d = self.int(format!("(ite {hit} {} {fd})", bv(0)));
                (k, d)
            }
            Expr::Case(en, c, data) => {
                let def = self.p.enums.iter().find(|d| &d.name == en);
                let (idx, ty) = def.and_then(|d| d.cases.iter().position(|x| &x.0 == c).map(|i| (i, d.cases[i].1))).unwrap_or((0, None));
                let d = match (data, ty) {
                    (Some(x), Some(t)) => {
                        let v = self.expr(ctx, x, Some(t));
                        (self.node, self.stmt) = (addr(&**x), addr(e));
                        self.land(ctx, &v, t.into(), &format!("data of {en}.{c}"));
                        v
                    }
                    _ => bv(0),
                };
                (bv(idx as i128), d)
            }
            _ => {
                self.too_big = true;
                (self.fresh_int(), self.fresh_int())
            }
        }
    }

    fn bind_enum(&mut self, name: &str, k: String, d: String) {
        let s = self.scopes.last_mut().unwrap();
        s.insert(name.to_string(), Var { term: k, ty: Ty::I64.into() });
        s.insert(format!("{name}#d"), Var { term: d, ty: Ty::I64.into() });
    }

    /// Run every arm of a match under its own case: the value each gives
    /// (None for an arm of statements), with its data bound.
    fn arms(&mut self, ctx: &Ctx, scrut: &Expr, arms: &[Arm], ctx_ty: Option<Ty>, leave: &mut Leave) -> Vec<(String, Option<String>)> {
        let (k, d) = self.enum_val(ctx, scrut);
        let mut out = vec![];
        for (i, a) in arms.iter().enumerate() {
            let is = self.boolean(format!("(= {k} {})", bv(i as i128)));
            let pc = self.boolean(format!("(and {} {is})", ctx.pc));
            let c = Ctx { pc };
            self.scopes.push(HashMap::new());
            if let Some(b) = &a.bind {
                let ty = self.p.enums.iter().flat_map(|e| e.cases.iter()).find(|x| x.0 == a.case).and_then(|x| x.1).unwrap_or(Ty::I64);
                self.scopes.last_mut().unwrap().insert(b.clone(), Var { term: d.clone(), ty: ty.into() });
            }
            let v = match &a.value {
                Some(v) => Some(self.expr(&c, v, ctx_ty)),
                None => {
                    self.block(&c, &a.body, leave);
                    None
                }
            };
            self.scopes.pop();
            out.push((is, v));
        }
        out
    }

    fn here(&mut self, ctx: &Ctx, leave: &Leave) -> Ctx {
        let mut gone = vec![];
        for f in [&leave.brk, &leave.cont, &leave.ret].into_iter().flatten() {
            if f != "false" {
                gone.push(f.clone());
            }
        }
        if gone.is_empty() {
            return ctx.clone();
        }
        let pc = self.boolean(format!("(and {} (not (or false {})))", ctx.pc, gone.join(" ")));
        Ctx { pc }
    }

    fn block(&mut self, ctx: &Ctx, body: &[Stmt], leave: &mut Leave) {
        self.scopes.push(HashMap::new());
        for s in body {
            if self.too_big {
                break;
            }
            let c = self.here(ctx, leave);
            self.stmt(&c, s, leave);
        }
        self.scopes.pop();
    }

    fn assign(&mut self, ctx: &Ctx, name: &str, v: String) {
        if let Some(old) = self.lookup(name) {
            let sort = if old.ty.base == Ty::Bool { "Bool".to_string() } else { format!("(_ BitVec {W})") };
            let t = self.def(&sort, format!("(ite {} {v} {})", ctx.pc, old.term));
            self.set(name, t);
        }
    }

    fn stmt(&mut self, ctx: &Ctx, s: &Stmt, leave: &mut Leave) {
        match s {
            Stmt::ListLet { name, elem, items, .. } => {
                for x in items {
                    let v = self.expr(ctx, x, Some(*elem));
                    (self.node, self.stmt) = (addr(x), addr(s));
                    self.land(ctx, &v, (*elem).into(), &format!("item of '{name}'"));
                }
            }
            Stmt::EnumLet { name, value, .. } => {
                let (k, d) = self.enum_val(ctx, value);
                self.bind_enum(name, k, d);
            }
            Stmt::Assign { name, value, .. } if self.is_enum(&Expr::Var(name.clone())) => {
                let (k, d) = self.enum_val(ctx, value);
                self.assign(ctx, name, k);
                self.assign(ctx, &format!("{name}#d"), d);
            }
            Stmt::MatchStmt { scrut, arms } => {
                self.arms(ctx, scrut, arms, None, leave);
            }
            Stmt::Let { name, ty, value: m @ Expr::Match(scrut, arms), .. } => {
                let vals = self.arms(ctx, scrut, arms, Some(ty.base), leave);
                // The value: an ite over the arms that give one; an arm that
                // left gives none, and the landing is only asked where none left.
                let mut v = bv(0);
                for (is, x) in vals.iter().rev() {
                    if let Some(x) = x {
                        v = self.int(format!("(ite {is} {x} {v})"));
                    }
                }
                let c = self.here(ctx, leave);
                (self.node, self.stmt) = (addr(m), addr(s));
                self.land(&c, &v, *ty, &format!("let '{name}'"));
                self.scopes.last_mut().unwrap().insert(name.clone(), Var { term: v, ty: *ty });
            }
            Stmt::SetAt { .. } | Stmt::Append { .. } | Stmt::ForEach { .. } | Stmt::Borrow { .. } | Stmt::Give { .. } | Stmt::SimdLet { .. } => {
                // Not modelled yet: the whole program is left to the other attacks.
                self.too_big = true;
            }
            Stmt::Let { name, ty, value, .. } => {
                let v = self.expr(ctx, value, Some(ty.base));
                (self.node, self.stmt) = (addr(value), addr(s));
                self.land(ctx, &v, *ty, &format!("let '{name}'"));
                self.scopes.last_mut().unwrap().insert(name.clone(), Var { term: v, ty: *ty });
            }
            Stmt::Assign { name, value, .. } => {
                let ty = self.lookup(name).map(|v| v.ty).unwrap_or(Ty::I64.into());
                let v = self.expr(ctx, value, Some(ty.base));
                (self.node, self.stmt) = (addr(value), addr(s));
                self.land(ctx, &v, ty, &format!("'{name}' ="));
                self.assign(ctx, name, v);
            }
            Stmt::Print { signed, value } => {
                let t = if *signed { Ty::I64 } else { Ty::U64 };
                self.call_at = addr(s);
                let x = self.expr(ctx, value, None);
                (self.node, self.stmt) = (addr(value), addr(s));
                self.land(ctx, &x, t.into(), "printed value");
            }
            Stmt::Return(e) => {
                let rt = leave.ret_ty.unwrap_or(Ty::I64.into());
                let v = self.expr(ctx, e, Some(rt.base));
                (self.node, self.stmt) = (addr(e), addr(s));
                self.land(ctx, &v, rt, "return");
                let prev_ret = leave.ret.clone().unwrap_or("false".into());
                let prev_val = leave.ret_val.clone().unwrap_or(bv(0));
                let sort = if rt.base == Ty::Bool { "Bool".to_string() } else { format!("(_ BitVec {W})") };
                let first = self.boolean(format!("(and {} (not {prev_ret}))", ctx.pc));
                leave.ret_val = Some(self.def(&sort, format!("(ite {first} {v} {prev_val})")));
                leave.ret = Some(self.boolean(format!("(or {prev_ret} {})", ctx.pc)));
            }
            Stmt::Break => {
                let b = leave.brk.clone().unwrap_or("false".into());
                leave.brk = Some(self.boolean(format!("(or {b} {})", ctx.pc)));
            }
            Stmt::Continue => {
                let c = leave.cont.clone().unwrap_or("false".into());
                leave.cont = Some(self.boolean(format!("(or {c} {})", ctx.pc)));
            }
            Stmt::Block { body, .. } => self.block(ctx, body, leave),
            Stmt::If { arms, otherwise } => {
                let mut none_before = "true".to_string();
                for (cond, body) in arms {
                    let c = self.expr(&Ctx { pc: format!("(and {} {none_before})", ctx.pc) }, cond, None);
                    let pc = self.boolean(format!("(and {} {none_before} {c})", ctx.pc));
                    self.block(&Ctx { pc }, body, leave);
                    none_before = self.boolean(format!("(and {none_before} (not {c}))"));
                }
                if let Some(body) = otherwise {
                    let pc = self.boolean(format!("(and {} {none_before})", ctx.pc));
                    self.block(&Ctx { pc }, body, leave);
                }
            }
            Stmt::For { perm, name, ty, start, stop, step, body } => {
                let (Expr::Lit(a), Expr::Lit(b)) = (start, stop) else { return };
                let st = step.unwrap_or(1);
                let mut vals = vec![];
                let mut i = *a;
                while (st > 0 && i <= *b) || (st < 0 && i >= *b) {
                    vals.push(i);
                    if vals.len() > MAX_TRIPS {
                        break;
                    }
                    i += st;
                }
                self.run_loop(ctx, leave, vals.len(), |enc, k, c, mut inner| {
                    enc.scopes.push(HashMap::new());
                    enc.scopes.last_mut().unwrap().insert(name.clone(), Var { term: bv(vals[k]), ty: (*ty).into() });
                    enc.block(c, body, &mut inner);
                    enc.scopes.pop();
                    inner
                });
                if *perm {
                    let last = vals.last().copied().unwrap_or(*a);
                    self.scopes.last_mut().unwrap().insert(name.clone(), Var { term: bv(last), ty: (*ty).into() });
                }
            }
            Stmt::While { cond, body } => {
                self.run_loop(ctx, leave, MAX_TRIPS, |enc, _, c, mut inner| {
                    let go = enc.expr(c, cond, None);
                    let pc = enc.boolean(format!("(and {} {go})", c.pc));
                    // Not going on counts as a break.
                    inner.brk = Some(enc.boolean(format!("(and {} (not {go}))", c.pc)));
                    enc.block(&Ctx { pc }, body, &mut inner);
                    inner
                });
            }
            Stmt::Loop { body } => {
                self.run_loop(ctx, leave, MAX_TRIPS, |enc, _, c, mut inner| {
                    enc.block(c, body, &mut inner);
                    inner
                });
            }
        }
    }

    /// Run a loop body `trips` times. Each trip sees the loop still going
    /// (no break yet, no return) and gets its own break/continue flags; a
    /// `return` inside it is carried out to the enclosing function.
    fn run_loop<F>(&mut self, ctx: &Ctx, leave: &mut Leave, trips: usize, mut trip: F)
    where
        F: FnMut(&mut Self, usize, &Ctx, Leave) -> Leave,
    {
        let mut broken = "false".to_string();
        for k in 0..trips {
            if self.too_big {
                return;
            }
            let ret = leave.ret.clone().unwrap_or("false".into());
            let pc = self.boolean(format!("(and {} (not {broken}) (not {ret}))", ctx.pc));
            let fresh = Leave { ret: leave.ret.clone(), ret_val: leave.ret_val.clone(), ret_ty: leave.ret_ty, ..Default::default() };
            let inner = trip(self, k, &Ctx { pc }, fresh);
            if let Some(b) = inner.brk {
                broken = self.boolean(format!("(or {broken} {b})"));
            }
            if leave.ret.is_some() {
                leave.ret = inner.ret;
                leave.ret_val = inner.ret_val;
            }
        }
    }
}

/// A program translated once: the definitions, and every site with its terms.
pub struct Encoded {
    head: String,
    raw: Vec<SiteRaw>,
    pub sites: Vec<Site>,
    pub inputs: Vec<String>,
}

fn bvw(v: i128, width: u32) -> String {
    if v >= 0 {
        format!("(_ bv{v} {width})")
    } else {
        format!("(bvneg (_ bv{} {width}))", -v)
    }
}

/// Translate a program, with its inputs ranging over 0..=input_max.
pub fn encode_with(p: &Program, input_max: u128) -> Option<Encoded> {
    let mut e = Enc { p, defs: String::new(), ndefs: 0, sites: vec![], node: 0, stmt: 0, call_at: 0, scopes: vec![HashMap::new()], too_big: false, depth: 0 };
    let mut decls = String::new();
    let mut inputs = vec![];
    for i in &p.inputs {
        let n = format!("in_{}", i.name);
        let _ = writeln!(decls, "(declare-const {n} (_ BitVec {W}))");
        let _ = writeln!(decls, "(assert (and (bvsle {} {n}) (bvsle {n} {})))", bv(0), bv(input_max as i128));
        e.scopes[0].insert(i.name.clone(), Var { term: n.clone(), ty: Ty::U64.into() });
        inputs.push(n);
    }
    let ctx = Ctx { pc: "true".into() };
    let mut leave = Leave::default();
    e.block(&ctx, &p.main, &mut leave);
    if e.too_big {
        return None;
    }
    let mut head = String::new();
    head.push_str("(set-logic QF_BV)\n(set-option :produce-models true)\n(set-option :rlimit 1500000)\n");
    head.push_str(&decls);
    head.push_str(&e.defs);
    let sites = e.sites.iter().map(|r| Site { what: r.what.clone(), kind: r.kind, node: r.node, stmt: r.stmt, fits: r.fits }).collect();
    Some(Encoded { head, raw: e.sites, sites, inputs })
}

/// A question about one site.
#[derive(Clone, Debug)]
pub enum Test {
    /// Can the site fail?
    Fails,
    /// Can its value fall outside this set?
    OutsideSet(Vec<i128>),
    /// Can its value fall outside this interval?
    OutsideInterval(i128, i128),
}

impl Encoded {
    fn ask(&self, s: &mut String, k: usize, cond: &str) {
        self.ask_any(s, &[(k, cond.to_string())]);
    }

    /// One question: does any of these sites, reached with nothing failing
    /// before it, meet its condition?
    fn ask_any(&self, s: &mut String, parts: &[(usize, String)]) {
        let vals = if self.inputs.is_empty() { "true".to_string() } else { self.inputs.join(" ") };
        let any: Vec<String> = parts
            .iter()
            .map(|(k, cond)| {
                let before = if *k == 0 { "true".to_string() } else { format!("ok{}", k - 1) };
                format!("(and {before} {} {cond})", self.raw[*k].reach)
            })
            .collect();
        let _ = writeln!(s, "(push 1)\n(assert (or false {}))\n(check-sat)\n(get-value ({vals}))\n(pop 1)", any.join(" "));
    }

    fn oks(&self, s: &mut String) {
        let mut ok = "true".to_string();
        for (k, r) in self.raw.iter().enumerate() {
            let _ = writeln!(s, "(define-fun ok{k} () Bool (and {ok} (=> {} (not {}))))", r.reach, r.fail);
            ok = format!("ok{k}");
        }
    }

    /// Every site, in order: can it fail, with nothing failing before it?
    pub fn aim_script(&self) -> String {
        let mut s = self.head.clone();
        self.oks(&mut s);
        for k in 0..self.raw.len() {
            let f = self.raw[k].fail.clone();
            self.ask(&mut s, k, &f);
        }
        s
    }

    /// The questions given, in order; each is about every instance of one
    /// claim at once. An `Outside` test on a site with no value term is asked
    /// as `Fails`.
    pub fn claim_script(&self, questions: &[(Vec<usize>, Test)]) -> String {
        let mut s = self.head.clone();
        self.oks(&mut s);
        for (ks, t) in questions {
            let parts: Vec<(usize, String)> = ks
                .iter()
                .map(|k| {
                    let r = &self.raw[*k];
                    let cond = match t {
                        Test::OutsideSet(vs) if r.width > 0 => {
                            let eqs: Vec<String> = vs.iter().map(|v| format!("(= {} {})", r.value, bvw(*v, r.width))).collect();
                            format!("(not (or false {}))", eqs.join(" "))
                        }
                        Test::OutsideInterval(lo, hi) if r.width > 0 => {
                            format!("(or (bvslt {} {}) (bvsgt {} {}))", r.value, bvw(*lo, r.width), r.value, bvw(*hi, r.width))
                        }
                        _ => r.fail.clone(),
                    };
                    (*k, cond)
                })
                .collect();
            self.ask_any(&mut s, &parts);
        }
        s
    }
}

/// Translate a program into one Z3 script: a `check-sat` per site, each
/// followed by the inputs' values when there is a model.
pub fn encode(p: &Program) -> Option<Query> {
    let e = encode_with(p, INPUT_MAX as u128)?;
    Some(Query { script: e.aim_script(), sites: e.sites.clone(), inputs: e.inputs.clone() })
}

/// What Z3 said about one site.
#[derive(Clone, Debug, PartialEq)]
pub enum Answer {
    /// A failing input (the inputs' values, in order).
    Fails(Vec<u64>),
    /// No input fails here (within the inputs' bounds).
    Never,
    /// Z3 gave up within its budget.
    Unknown,
}

/// The answers Z3 gave, site by site; shorter than the sites when Z3 was
/// stopped part way.
pub fn parse_answers(out: &str, nsites: usize, ninputs: usize) -> Vec<Answer> {
    let mut answers = vec![];
    // Any error but "no model after unsat" means the script itself is wrong:
    // no answer counts then.
    if out.lines().any(|l| l.trim_start().starts_with("(error") && !l.contains("model is not available")) {
        return answers;
    }
    let mut lines = out.lines().peekable();
    while answers.len() < nsites {
        let Some(l) = lines.next() else { break };
        let l = l.trim();
        match l {
            "sat" => {
                // the get-value answer: may span lines, until the parens close
                let mut text = String::new();
                let mut depth = 0i32;
                for l2 in lines.by_ref() {
                    text.push_str(l2);
                    text.push(' ');
                    depth += l2.matches('(').count() as i32 - l2.matches(')').count() as i32;
                    if depth <= 0 {
                        break;
                    }
                }
                let mut vals = vec![];
                for part in text.split("#x").skip(1) {
                    let hex: String = part.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                    vals.push(u128::from_str_radix(&hex, 16).unwrap_or(0) as u64);
                }
                for part in text.split("#b").skip(1) {
                    let bin: String = part.chars().take_while(|c| *c == '0' || *c == '1').collect();
                    vals.push(u128::from_str_radix(&bin, 2).unwrap_or(0) as u64);
                }
                answers.push(if vals.len() == ninputs { Answer::Fails(vals) } else { Answer::Unknown });
            }
            "unsat" | "unknown" => {
                let a = if l == "unsat" { Answer::Never } else { Answer::Unknown };
                // the get-value that follows fails; skip its error line
                if let Some(n) = lines.peek() {
                    if n.trim_start().starts_with("(error") {
                        lines.next();
                    }
                }
                answers.push(a);
            }
            _ => {}
        }
    }
    answers
}
