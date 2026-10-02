//! Random valid programs. Valid by construction: every name is declared
//! before use, every type matches, every loop ends, and every operation
//! stays inside its type unless the generator deliberately leaves it
//! unguarded (`Config::risk`), so that both engines are also compared on
//! checks that fire.
//!
//! The generator keeps its own interval per value — a deliberately simple
//! bookkeeping that is never shown to either engine. Where an operation
//! could leave its type, an operand is folded into range with
//! `lo + (e mod w)`, which is a plain CyborgPL guardrail.

use crate::ast::*;
use crate::printer::{printer, strict_printer};
use crate::rng::Rng;

#[derive(Clone, Debug)]
pub struct Config {
    /// 1..=6: how many functions, statements, and how deep.
    pub size: u32,
    /// Read outside values (environment-variable lengths) at the start of MAIN.
    pub inputs: u32,
    /// Operations left unguarded, per thousand.
    pub risk: u64,
    /// Write `xx` (CCA does not generate it yet).
    pub pow: bool,
    /// Keep `for` loops whose last step would pass the counter type's edge
    /// (CCA refuses them; finding sent 2026-09-23).
    pub range_edge: bool,
    /// `shl`, `not.bit` and the wrapping operators on any typed operand, not
    /// only a name (owner, 2026-09-23, 88da73f; CCA refuses it as of 61a8554).
    pub width_from_context: bool,
    /// Give some names, parameters and returns a range type.
    pub ranges: bool,
    /// Aim at the 64-bit edges: mostly int64/uint64, edge literals, call
    /// results known only by their type, comparisons across types
    /// (layer 3's wrong-proof family, compiler claude 2026-09-23).
    pub edge: bool,
    /// Give the operands of shifts, bitwise and wrapping operators a size
    /// with `to.T[…]` unless they are names or literals: CCA says "not yet"
    /// to them on an exact answer (2026-09-23).
    pub sized_bits: bool,
    /// Write recursive functions: a range-typed counter parameter that
    /// counts down to 0 (or up to its top) on every call.
    pub recursion: bool,
    /// Attack (a) of the Precog oracle: narrow every input at the door of
    /// MAIN into a small typed value, so that the whole input domain (at most
    /// this many combinations) can be run. 0 = off.
    pub exhaust: u64,
    /// Lists (design/lists.md): literals, positions (check sites), `change
    /// 'xs'[p]`, `list.append`, `list.length`, `for 'i', 'x' in lend 'xs'`.
    pub lists: bool,
    /// Use a `for … in lend` item only through `to.T['x']`: the oracle gives
    /// the item no width for `+~` or `not.bit` (sent 2026-09-29).
    pub item_through_to: bool,
    /// Use a `match` arm's data name only through `to.T['d']`: the oracle
    /// gives it no width for `not.bit` or the wrapping operators (sent
    /// 2026-09-29).
    pub data_through_to: bool,
    /// Functions that hand back a borrow (`-> lent@x list[T]`, 46faa74), and
    /// chains of them in MAIN followed by a `give`, an append or a change of
    /// one of the lists.
    pub borrows: bool,
    /// Per mille of programs with borrows that get one borrow mistake on
    /// purpose (a borrow of a local, the wrong group, an unsaid borrow, or a
    /// list changed while a borrow of it is still used); both engines must
    /// refuse those (`Program::expect_refused`).
    pub borrow_wrong: u64,
    /// Enums with integer data, `==` on them, and `match` as a statement
    /// and as a value (design/enums.md; 23ab20b).
    pub enums: bool,
    /// Per mille of programs written as `#strict` files.
    pub strict: u64,
    /// simd vectors (design/simd.md): `simd.fill`, lane-wise operators kept
    /// inside the lane type, comparisons folded with `all` / `any`, and
    /// `max` / `min` / `sum`.
    pub simd: bool,
    /// Read single lanes, `'v'[k]` (CCA read wrong values, sent 2026-09-29).
    pub simd_lanes: bool,
    /// The wrapping operators on vectors (the oracle refuses them, sent
    /// 2026-09-29); without them a lane that could leave its type is made
    /// by a bitwise operator instead.
    pub simd_wrap: bool,
    /// deci values, their arithmetic and conversions, and text into
    /// numbers (design/types.md, design/conversion.md; owner "yea",
    /// 2026-09-30), watched through printed integers.
    pub deci: bool,
    /// The `to.useful.…` forms (CCA does not take the 2026-09-30 spelling yet).
    pub deci_useful: bool,
    /// `for !c` / `for !c seq` on some range loops (owner, 2026-10-01).
    pub no_cheat: bool,
    /// `brain16` (owner, 2026-10-02): its own family, each step rounded,
    /// seen through conversions; never inside a `check`.
    pub brain16: bool,
    /// `mod` on brain16 (its sign for bins is open with the owner, 2026-10-02).
    pub brain_mod: bool,
    /// The roundings past `.floor`: `div.trunc`, `div.ceil`, `div.near` and
    /// their `mod`s on integers; `.floor`, `.ceil`, `.trunc` and their
    /// `mod`s on decis and brain16 (owner, 2026-10-02, 692a444, 5a12c07).
    pub roundings: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config { size: 3, inputs: 0, risk: 5, pow: false, range_edge: false, width_from_context: false, ranges: true, edge: false, sized_bits: true, recursion: true, exhaust: 0, lists: true, item_through_to: false, data_through_to: false, borrows: true, borrow_wrong: 250, enums: true, strict: 200, simd: true, simd_lanes: true, simd_wrap: true, deci: true, deci_useful: true, no_cheat: true, brain16: true, brain_mod: true, roundings: true }
    }
}

#[derive(Clone, Debug)]
struct VarInfo {
    name: String,
    ty: Ty,
    iv: Iv,
    assignable: bool,
    /// Set for a list: `ty` and `iv` are then its items'.
    list: Option<ListInfo>,
    /// The item of a `for … in lend` (see `Config::item_through_to`).
    item: bool,
    /// A `match` arm's data (see `Config::data_through_to`).
    data: bool,
    /// Its range is known to the generator only: it comes from a self-call
    /// (whose result only an induction over the calls could bound), or from
    /// a value that used such a name, with no range type to say it. Never
    /// used in a `check` statement or block (seed 137, 2026-09-29).
    unseen: bool,
    /// Set for an enum value: the index of its enum in `Gen::enums`.
    en: Option<usize>,
    /// Set for a simd vector: its lanes (`ty` and `iv` are then a lane's).
    simd: Option<usize>,
}

#[derive(Clone, Debug)]
struct EnumGen {
    def: EnumDef,
    /// What each case's data can be (the generator's own promise).
    ivs: Vec<Option<Iv>>,
}

#[derive(Clone, Debug)]
struct ListInfo {
    /// Its length is never below this (nothing is ever removed) …
    lo: u64,
    /// … and never above this: appends stop there.
    cap: u64,
    /// The most items the appends generated so far can have made.
    len_now: u64,
    mutable: bool,
    /// `mult` where it was declared: an append runs mult / decl_mult times
    /// per life of the list.
    decl_mult: u64,
    /// Being walked by a `for … in lend`: nothing may change it.
    frozen: u32,
    /// The walk's position name, while inside one that has it.
    walk_pos: Option<String>,
    /// Walked with `lend.mut`: while the walk lasts nothing else may use it,
    /// not even to read (Rust-style borrowing; seed 9512, 2026-09-30).
    lent_mut: bool,
}

#[derive(Clone, Debug)]
struct Sig {
    name: String,
    params: Vec<(String, Ty, Iv)>,
    ret: Ty,
    ret_iv: Iv,
    /// The return has a range type, so a caller sees what it can be.
    ret_ranged: bool,
    cost: u64,
}

/// A borrow function as its callers see it.
#[derive(Clone, Debug)]
struct BSig {
    name: String,
    arity: usize,
    /// The parameters its result may borrow (by the signature's group, which
    /// the generator makes equal to what the body can return).
    deps: Vec<usize>,
    wrong: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum LoopKind {
    None,
    Loop,
}

struct Gen {
    rng: Rng,
    cfg: Config,
    scopes: Vec<Vec<VarInfo>>,
    sigs: Vec<Sig>,
    callable: usize,
    fresh: u32,
    in_loop: LoopKind,
    in_fn: Option<(Ty, Iv)>,
    /// Product of the trip counts of the loops around the current point.
    mult: u64,
    /// Work units left for the current function or MAIN.
    budget: u64,
    spent: u64,
    risky: bool,
    /// Inside a `check` block: no operation is left unguarded here, since
    /// CCA must (rightly) refuse a `check` it cannot prove.
    in_check: u32,
    /// The statement being built reads or writes a list by position: not a
    /// `check` statement, since a guarded position is a fact about the
    /// list's length that Precog need not see.
    positioned: bool,
    bsigs: Vec<BSig>,
    belem: Ty,
    enums: Vec<EnumGen>,
    /// The program holds something my exact evaluator does not model.
    unevaluable: bool,
    /// Functions written as source.
    extra_fns: Vec<String>,
    /// `m1 [lent.mut 'a', lent 'b']` is defined: one call may not lend one
    /// list both ways (owner: a, 38c91e7).
    mutator: bool,
    /// A `#strict` file: the whole program under the generator's `check`
    /// rules (no risk, no positions, nothing unseen), since every site must
    /// be proven in its own function, and promised ranges are whole types.
    strict: bool,
}

const MAIN_BUDGET: u64 = 20_000;
const FN_BUDGET: u64 = 400;

pub fn generate(seed: u64, cfg: &Config) -> Program {
    let mut root = Rng::new(seed);
    let mut prng = root.split();
    let strict = cfg.strict > 0 && root.below(1000) < cfg.strict;
    let printer = if strict { strict_printer(&mut prng) } else { printer(&mut prng) };
    let loose = root.chance(50);
    let mut g = Gen {
        rng: root.split(),
        cfg: cfg.clone(),
        scopes: vec![],
        sigs: vec![],
        callable: 0,
        fresh: 0,
        in_loop: LoopKind::None,
        in_fn: None,
        mult: 1,
        budget: 0,
        spent: 0,
        risky: false,
        in_check: 0,
        positioned: false,
        bsigs: vec![],
        belem: Ty::I32,
        enums: vec![],
        strict,
        unevaluable: false,
        extra_fns: vec![],
        mutator: false,
    };
    if strict {
        g.in_check = 1;
    }
    if cfg.enums && g.rng.chance(45) {
        for k in 0..1 + g.rng.below(2) as usize {
            g.enum_def(k);
        }
    }
    let nfns = g.rng.below(cfg.size as u64 + 1) as usize;
    let mut funcs = vec![];
    for i in 0..nfns {
        funcs.push(g.function(i));
    }
    // Borrow functions, and whether this program gets a mistake on purpose.
    let mut bfuncs = vec![];
    let mut wrong = None;
    if cfg.borrows && g.rng.chance(35) {
        g.belem = g.int_ty();
        let n = 1 + g.rng.below(4) as usize;
        let mistake = g.rng.below(1000) < cfg.borrow_wrong;
        let bad = if mistake && g.rng.chance(50) { Some(g.rng.below(n as u64) as usize) } else { None };
        for i in 0..n {
            bfuncs.push(g.borrow_fn(i, bad == Some(i)));
        }
        if g.rng.chance(50) {
            let t = g.belem.name();
            g.extra_fns.push(format!("fn m1 [lent.mut 'a': list[{t}], lent 'b': list[{t}]] {{\n    if list.length[lend 'b'] > 0 {{\n        list.append[lend.mut 'a', 1];\n    }}\n}}\n"));
            g.mutator = true;
        }
        if mistake {
            wrong = Some(bad);
        }
    }
    // MAIN
    g.callable = g.sigs.len();
    g.in_fn = None;
    g.budget = MAIN_BUDGET;
    g.spent = 0;
    g.mult = 1;
    g.scopes = vec![vec![]];
    let mut inputs = vec![];
    let mut main = vec![];
    let mut room = cfg.exhaust;
    for k in 0..cfg.inputs {
        let name = format!("in{}", k + 1);
        if cfg.exhaust > 0 {
            // The raw input is never visible: only its narrowed door value.
            let (door, domain) = g.door(&name, room);
            room /= domain;
            inputs.push(Input { name: name.clone(), var: format!("CHK_IN{}", k + 1), domain: Some(domain) });
            main.push(door);
        } else {
            inputs.push(Input { name: name.clone(), var: format!("CHK_IN{}", k + 1), domain: None });
            g.declare(name, Ty::U64, Ty::U64.range(), false);
        }
    }
    // The mistake goes first in MAIN, where it always runs (the oracle
    // checks borrows only on the paths it runs, sent 2026-09-29).
    if let Some(bad) = wrong {
        main.push(g.episode(Some(bad)));
    }
    // A few typed names first, so comparisons always have something typed.
    for _ in 0..1 + g.rng.below(3) {
        main.push(g.let_stmt());
    }
    let n = g.stmt_count();
    main.extend(g.stmts(n, g.depth()));
    Program { seed, printer, inputs, funcs, bfuncs, extra_fns: g.extra_fns.clone(), enums: g.enums.iter().map(|e| e.def.clone()).collect(), main, loose, expect_refused: wrong.is_some(), strict, unevaluable: g.unevaluable }
}

fn isqrt(n: i128) -> i128 {
    let mut x = (n as f64).sqrt() as i128;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}

fn corners(ia: Iv, ib: Iv, f: impl Fn(i128, i128) -> Option<i128>) -> Option<Iv> {
    let vs = [f(ia.lo, ib.lo), f(ia.lo, ib.hi), f(ia.hi, ib.lo), f(ia.hi, ib.hi)];
    let mut lo = i128::MAX;
    let mut hi = i128::MIN;
    for v in vs {
        let v = v?;
        lo = lo.min(v);
        hi = hi.max(v);
    }
    Some(Iv::new(lo, hi))
}

fn floor_div(a: i128, b: i128) -> i128 {
    let q = a / b;
    if (a % b != 0) && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

fn is_literal_only(e: &Expr) -> bool {
    match e {
        Expr::Lit(_) | Expr::Bool(_) => true,
        Expr::Var(_) | Expr::Call(..) | Expr::To(..) | Expr::At(..) | Expr::Len(_) | Expr::Lend(_) | Expr::Case(..) | Expr::Match(..) | Expr::SimdFill(_) | Expr::SimdFold(..) | Expr::ToEnum(..) | Expr::Num(_) | Expr::Text(_) | Expr::Conv(..) | Expr::StrBytes(_) | Expr::Src(_) => false,
        Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => is_literal_only(a) && is_literal_only(b),
        Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) => is_literal_only(a),
    }
}

impl Gen {
    fn name(&mut self, prefix: &str) -> String {
        self.fresh += 1;
        format!("{prefix}{}", self.fresh)
    }

    fn stmt_count(&mut self) -> usize {
        2 + self.rng.below(self.cfg.size as u64 + 2) as usize
    }

    fn depth(&self) -> u32 {
        1 + self.cfg.size / 2
    }

    fn expr_depth(&mut self) -> u32 {
        self.rng.below(2 + self.cfg.size as u64 / 2) as u32 + 1
    }

    fn declare(&mut self, name: String, ty: Ty, iv: Iv, assignable: bool) {
        self.scopes.last_mut().unwrap().push(VarInfo { name, ty, iv, assignable, list: None, item: false, data: false, unseen: false, en: None, simd: None });
    }

    /// Plain values in scope (never a list; inside a `check` block, never
    /// an unseen one).
    fn vars(&self) -> impl Iterator<Item = &VarInfo> {
        let in_check = self.in_check > 0;
        self.scopes.iter().flatten().filter(move |v| v.list.is_none() && v.en.is_none() && v.simd.is_none() && !(in_check && v.unseen))
    }

    /// Whether `v` is read only through `to.T[…]` (an engine gives it no width).
    fn through_to(&self, v: &VarInfo) -> bool {
        (self.cfg.item_through_to && v.item) || (self.cfg.data_through_to && v.data)
    }

    fn simd_vars(&self, t: Option<Ty>) -> Vec<VarInfo> {
        self.scopes.iter().flatten().filter(|v| v.simd.is_some() && t.map_or(true, |t| v.ty == t)).cloned().collect()
    }

    fn enum_vars(&self) -> Vec<VarInfo> {
        self.scopes.iter().flatten().filter(|v| v.en.is_some()).cloned().collect()
    }

    /// Whether `e` reads an unseen name.
    fn uses_unseen(&self, e: &Expr) -> bool {
        // A call's result is unseen unless its return has a range type: the
        // generator's promise for it is its own (seed 3118, 2026-09-29).
        if self.calls_unseen(e) {
            return true;
        }
        let unseen = |n: &str| self.scopes.iter().flatten().rev().find(|v| v.name == n).map_or(false, |v| v.unseen);
        fn walk(e: &Expr, f: &dyn Fn(&str) -> bool) -> bool {
            match e {
                Expr::Var(n) => f(n),
                // A length's bounds and an item's range are the generator's.
                // A lane's range is the generator's too.
                Expr::Len(_) | Expr::At(..) | Expr::Match(..) | Expr::SimdFold(..) => true,
                Expr::SimdFill(a) => walk(a, f),
                Expr::ToEnum(_, a, b) => walk(a, f) || walk(b, f),
                // a conversion's result is the generator's guess at most
                Expr::Conv(..) | Expr::StrBytes(_) | Expr::Src(_) => true,
                Expr::Num(_) | Expr::Text(_) => false,
                Expr::Case(_, _, d) => d.as_ref().map_or(false, |d| walk(d, f)),
                Expr::Lit(_) | Expr::Bool(_) | Expr::Lend(_) => false,
                Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => walk(a, f) || walk(b, f),
                Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) => walk(a, f),
                Expr::Call(_, args) => args.iter().any(|a| walk(a, f)),
            }
        }
        walk(e, &unseen)
    }

    fn calls_unseen(&self, e: &Expr) -> bool {
        match e {
            // In a strict file every promise is the whole type: nothing unseen.
            Expr::Call(f, args) => (!self.strict && !self.sigs.iter().any(|s| &s.name == f && s.ret_ranged)) || args.iter().any(|a| self.calls_unseen(a)),
            Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => self.calls_unseen(a) || self.calls_unseen(b),
            Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) | Expr::At(_, a) => self.calls_unseen(a),
            _ => false,
        }
    }

    fn mark_unseen(&mut self, name: &str) {
        if let Some(v) = self.scopes.iter_mut().flatten().rev().find(|v| v.name == name) {
            v.unseen = true;
        }
    }

    fn lists(&self) -> Vec<VarInfo> {
        self.scopes.iter().flatten().filter(|v| v.list.as_ref().map_or(false, |l| !l.lent_mut)).cloned().collect()
    }

    fn list_mut(&mut self, name: &str) -> &mut ListInfo {
        self.scopes.iter_mut().flatten().rev().find(|v| v.name == name).and_then(|v| v.list.as_mut()).unwrap()
    }

    fn vars_of(&self, t: Ty) -> Vec<VarInfo> {
        self.vars().filter(|v| v.ty == t && !self.through_to(v)).cloned().collect()
    }

    fn int_ty(&mut self) -> Ty {
        if self.cfg.edge && self.rng.chance(70) {
            return if self.rng.chance(50) { Ty::I64 } else { Ty::U64 };
        }
        *self.rng.pick(&INTS)
    }

    /// A sub-interval of `t` to promise for a name, or the whole type.
    fn ghost(&mut self, t: Ty) -> Iv {
        let r = t.range();
        if self.strict {
            // Facts from callers and other bodies are not used in a strict file.
            return r;
        }
        if self.cfg.edge && self.rng.chance(60) {
            return r;
        }
        match self.rng.below(4) {
            0 => r,
            1 => {
                // near an edge
                let w = self.rng.range(1, 64.min(r.width()));
                if self.rng.chance(50) {
                    Iv::new(r.lo, r.lo + w - 1)
                } else {
                    Iv::new(r.hi - w + 1, r.hi)
                }
            }
            _ => {
                let a = self.rng.range(r.lo.max(-1000), r.hi.min(1000));
                let b = self.rng.range(r.lo.max(-1000), r.hi.min(1000));
                Iv::new(a.min(b), a.max(b))
            }
        }
    }

    /// Maybe a range type over `t`, and the interval it allows.
    fn maybe_range(&mut self, t: Ty) -> (TyR, Option<Iv>) {
        if t == Ty::Bool || !self.cfg.ranges || !self.rng.chance(20) {
            return (t.into(), None);
        }
        let r = t.range();
        let near = |g: &mut Gen, edge: i128| -> i128 {
            let v = if g.rng.chance(50) { edge } else { g.rng.range(r.lo.max(-200), r.hi.min(200)) };
            v.clamp(r.lo, r.hi)
        };
        // The whole type written long is allowed (owner, 2026-09-23): rarely.
        if self.rng.chance(5) {
            let lo = if self.rng.chance(50) { None } else { Some(r.lo) };
            let hi = if self.rng.chance(50) { None } else { Some(r.hi) };
            return (TyR { base: t, range: Some((lo, hi)) }, Some(r));
        }
        let (lo, hi) = match self.rng.below(3) {
            0 => {
                let e1 = if self.rng.chance(50) { 0 } else { 1 };
                let a = near(self, e1);
                let e2 = if self.rng.chance(50) { 100 } else { r.hi };
                let b = near(self, e2);
                (Some(a.min(b)), Some(a.max(b)))
            }
            1 => (Some(near(self, 1).max(r.lo + 1)), None),
            _ => (None, Some(near(self, 100).min(r.hi - 1))),
        };
        if lo.is_none() && hi.is_none() {
            return (t.into(), None);
        }
        let iv = Iv::new(lo.unwrap_or(r.lo), hi.unwrap_or(r.hi));
        (TyR { base: t, range: Some((lo, hi)) }, Some(iv))
    }

    /// A door: `in` narrowed into a small typed value whose every value is
    /// reached by some length 0..domain-1, with domain <= room. Returns the
    /// `let` and the domain.
    fn door(&mut self, input: &str, room: u64) -> (Stmt, u64) {
        let name = self.name("d");
        let inp = || Box::new(Expr::Var(input.to_string()));
        let modulo = |m: i128| Expr::Bin(BinOp::Mod, inp(), Box::new(Expr::Lit(m)));
        let kind = if room >= 256 { self.rng.below(4) } else { 2 + self.rng.below(2) };
        let (ty, value, iv, m): (TyR, Expr, Iv, u64) = match kind {
            0 => (
                Ty::I8.into(),
                Expr::To(Ty::I8, Box::new(Expr::Bin(BinOp::Sub, Box::new(Expr::To(Ty::I16, Box::new(modulo(256)))), Box::new(Expr::Lit(128))))),
                Ty::I8.range(),
                256,
            ),
            1 => (Ty::U8.into(), Expr::To(Ty::U8, Box::new(modulo(256))), Ty::U8.range(), 256),
            2 if room >= 2 => {
                // a small range type, anywhere in its base type
                let t = self.int_ty();
                let m = self.rng.range(2, (room.min(40)) as i128);
                let r = t.range();
                let lo = if self.rng.chance(40) { self.rng.range(r.lo, r.hi - (m - 1)) } else { self.rng.range(r.lo.max(-60), (r.hi - (m - 1)).min(60)) };
                let base = Expr::To(t, Box::new(modulo(m)));
                let value = if lo == 0 {
                    base
                } else if lo > 0 {
                    Expr::Bin(BinOp::Add, Box::new(base), Box::new(Expr::Lit(lo)))
                } else {
                    Expr::Bin(BinOp::Sub, Box::new(base), Box::new(Expr::Lit(-lo)))
                };
                (TyR { base: t, range: Some((Some(lo), Some(lo + m - 1))) }, value, Iv::new(lo, lo + m - 1), m as u64)
            }
            _ => (Ty::Bool.into(), Expr::Cmp(CmpOp::Eq, Box::new(modulo(2)), Box::new(Expr::Lit(1))), Iv::new(0, 1), 2),
        };
        self.declare(name.clone(), ty.base, iv, false);
        (Stmt::Let { tier: Tier::Default, name, mutability: Mutability::Plain, ty, value }, m)
    }

    // ------------------------------------------------------------ functions

    fn function(&mut self, index: usize) -> Func {
        let name = format!("f{}", index + 1);
        let nparams = self.rng.below(4) as usize;
        let mut params = vec![];
        let mut param_tys = vec![];
        for k in 0..nparams {
            let t = if self.rng.chance(10) { Ty::Bool } else { self.int_ty() };
            let (tr, rng_iv) = self.maybe_range(t);
            let iv = if t == Ty::Bool { Iv::new(0, 1) } else { rng_iv.unwrap_or_else(|| self.ghost(t)) };
            params.push((format!("p{}", k + 1), t, iv));
            param_tys.push(tr);
        }
        // A recursive function: its first parameter is a range-typed counter
        // that every self-call moves one step toward the base case, so each
        // call ends. Downward to 0, or upward to the top — the upward shape is
        // one CCA's induction must not prove by mistake (compiler claude).
        let rec = if self.cfg.recursion && self.rng.chance(30) {
            let top = self.rng.range(1, 10);
            let up = self.rng.chance(30);
            let ct = *self.rng.pick(&[Ty::I32, Ty::I64, Ty::U8, Ty::U32]);
            params.insert(0, ("pr".to_string(), ct, Iv::new(0, top)));
            param_tys.insert(0, TyR { base: ct, range: Some((Some(0), Some(top))) });
            Some((ct, top, up))
        } else {
            None
        };
        let ret = if self.rng.chance(10) { Ty::Bool } else { self.int_ty() };
        let (ret_tr, ret_range) = self.maybe_range(ret);
        let ret_iv = if ret == Ty::Bool { Iv::new(0, 1) } else { ret_range.unwrap_or_else(|| self.ghost(ret)) };
        self.callable = index;
        self.in_fn = Some((ret, ret_iv));
        self.budget = FN_BUDGET;
        self.spent = 0;
        self.mult = 1;
        self.scopes = vec![vec![]];
        for (n, t, iv) in &params {
            self.declare(n.clone(), *t, *iv, false);
            // In a recursive function a parameter's range (other than the
            // counter's range type) holds only by induction over the calls
            // (seed 1520, 2026-09-29).
            if rec.is_some() && n != "pr" {
                self.mark_unseen(n);
            }
        }
        let mut body = vec![];
        if let Some((_, top, up)) = rec {
            let pr = || Box::new(Expr::Var("pr".into()));
            let (base_cond, step) = if up {
                (Expr::Cmp(CmpOp::Ge, pr(), Box::new(Expr::Lit(top))), BinOp::Add)
            } else {
                (Expr::Cmp(CmpOp::Eq, pr(), Box::new(Expr::Lit(0))), BinOp::Sub)
            };
            let (base, _) = self.value_in(ret, ret_iv);
            body.push(Stmt::If { arms: vec![(base_cond, vec![Stmt::Return(base)])], otherwise: None });
            // The self-call, its value kept in a name the rest may use.
            let mut args = vec![Expr::Bin(step, pr(), Box::new(Expr::Lit(1)))];
            for (_, pt, piv) in params.iter().skip(1) {
                let (a, _) = if *pt == Ty::Bool { (self.bool_expr(1), Iv::new(0, 1)) } else { self.value_in(*pt, *piv) };
                args.push(a);
            }
            let rname = self.name("r");
            body.push(Stmt::Let { tier: Tier::Default, name: rname.clone(), mutability: Mutability::Plain, ty: ret.into(), value: Expr::Call(name.clone(), args) });
            self.declare(rname.clone(), ret, ret_iv, false);
            self.mark_unseen(&rname);
        }
        let n = self.stmt_count() / 2 + 1;
        body.extend(self.stmts(n, self.depth().saturating_sub(1).max(1)));
        let (e, _) = self.value_in(ret, ret_iv);
        body.push(Stmt::Return(e));
        // A recursive function runs its body once per step of its counter.
        let calls = rec.map_or(1, |(_, top, _)| top as u64 + 1);
        self.sigs.push(Sig { name: name.clone(), params: params.clone(), ret, ret_iv, ret_ranged: ret_tr.range.is_some(), cost: self.spent.max(1).saturating_mul(calls) });
        Func { name, params: params.into_iter().zip(param_tys).map(|((name, _, _), ty)| Param { name, ty }).collect(), ret: ret_tr, body }
    }

    // ------------------------------------------------------------ statements

    fn stmts(&mut self, n: usize, depth: u32) -> Vec<Stmt> {
        let mut out = vec![];
        for _ in 0..n {
            if self.spent >= self.budget {
                break;
            }
            self.spent += self.mult;
            out.push(self.stmt(depth));
        }
        out
    }

    fn block(&mut self, n: usize, depth: u32) -> Vec<Stmt> {
        self.scopes.push(vec![]);
        let b = self.stmts(n, depth);
        self.scopes.pop();
        b
    }

    fn stmt(&mut self, depth: u32) -> Stmt {
        let assignable = self.vars().any(|v| v.assignable);
        let nested = depth > 0;
        let w = [
            6,                                      // let
            if assignable { 6 } else { 0 },         // assign
            if nested { 3 } else { 0 },             // if
            if nested { 3 } else { 0 },             // for
            if nested { 1 } else { 0 },             // while
            if nested { 1 } else { 0 },             // loop
            4,                                      // print
            if nested { 1 } else { 0 },             // block
            if self.cfg.lists { 2 } else { 0 },     // list let
            if self.settable().is_empty() { 0 } else { 2 },   // change 'xs'[p]
            if self.appendable().is_empty() { 0 } else { 2 }, // list.append
            if nested && !self.walkable().is_empty() { 4 } else { 0 }, // for … in lend
            if self.in_fn.is_none() && self.bsigs.iter().any(|b| !b.wrong) { 2 } else { 0 }, // borrows
            if self.enums.is_empty() { 0 } else { 2 },                               // enum let
            if self.enum_vars().iter().any(|v| v.assignable) { 1 } else { 0 },       // enum change
            if nested && !self.enums.is_empty() { 2 } else { 0 },                    // match statement
            if self.cfg.simd { 2 } else { 0 },                                       // simd let
            if self.simd_vars(None).iter().any(|v| v.assignable) { 1 } else { 0 },   // simd change
            if self.cfg.simd_lanes && self.in_check == 0 && self.simd_vars(None).iter().any(|v| v.assignable) { 1 } else { 0 }, // lane change
            if self.cfg.deci { 2 } else { 0 },                                       // deci and conversions
            if self.in_check == 0 && !self.lists().is_empty() { 6 } else { 0 },    // loop over a list's positions
            // no Precog domain yet: every site is a run-time check, which a `check` must not hold
            if self.cfg.brain16 && self.in_check == 0 { 2 } else { 0 },             // brain16
        ];
        match self.rng.weighted(&w) {
            0 => self.let_stmt(),
            1 => self.assign_stmt(),
            2 => self.if_stmt(depth),
            3 => self.for_stmt(depth),
            4 => self.counted_loop(depth, false),
            5 => self.counted_loop(depth, true),
            8 => self.list_let(),
            9 => self.set_at(),
            10 => self.append(),
            11 => self.for_each(depth),
            12 => self.episode(None),
            13 => self.enum_let(),
            14 => self.enum_assign(),
            15 => self.match_stmt(depth),
            16 => self.simd_let(),
            17 => self.simd_assign(),
            18 => self.lane_write(),
            19 => self.deci_episode(),
            20 => self.list_loop(),
            21 => self.brain_episode(),
            6 if self.rng.chance(8) => self.min_by_minus_one(),
            6 => {
                let t = self.int_ty();
                let (e, _) = self.int_expr_typed(t);
                Stmt::Print { signed: t.signed(), value: e }
            }
            _ => {
                let tier = match self.rng.below(10) {
                    0..=3 => Tier::Check,
                    4 => Tier::Nocheck,
                    5 => Tier::NoChecks,
                    _ => Tier::Default,
                };
                let n = self.stmt_count() / 2 + 1;
                // No risk inside a vouch either: what it vouches for must hold.
                let guarded = tier != Tier::Default;
                if guarded {
                    self.in_check += 1;
                }
                let body = self.block(n, depth - 1);
                if guarded {
                    self.in_check -= 1;
                }
                Stmt::Block { tier, body }
            }
        }
    }

    // ------------------------------------------------------------ lists

    /// Lists whose items may be read by position without a guard failing:
    /// never empty, or walked right now with a position name.
    fn readable(&self, t: Option<Ty>) -> Vec<VarInfo> {
        if self.in_check > 0 {
            return vec![];
        }
        self.lists().into_iter().filter(|v| t.map_or(true, |t| v.ty == t)).filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.lo >= 1 || l.walk_pos.is_some()
        }).collect()
    }

    fn settable(&self) -> Vec<VarInfo> {
        if self.in_check > 0 {
            return vec![];
        }
        self.lists().into_iter().filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.mutable && l.frozen == 0 && l.lo >= 1
        }).collect()
    }

    fn appendable(&self) -> Vec<VarInfo> {
        self.lists().into_iter().filter(|v| {
            let l = v.list.as_ref().unwrap();
            l.mutable && l.frozen == 0 && l.len_now + (self.mult / l.decl_mult.max(1)).max(1) <= l.cap
        }).collect()
    }

    fn walkable(&self) -> Vec<VarInfo> {
        let left = self.trips_allowed();
        self.lists().into_iter().filter(|v| v.list.as_ref().unwrap().cap <= left.max(1)).collect()
    }

    fn list_let(&mut self) -> Stmt {
        if self.rng.chance(30) {
            return self.list_fill();
        }
        let elem = self.int_ty();
        let n = self.rng.below(5);
        let mutable = n == 0 || self.rng.chance(60);
        let iv = self.ghost(elem);
        let mut items = vec![];
        self.risky = false;
        self.positioned = false;
        // The items take the list's item type as their context, as a
        // declaration's value does (owner, 62186f1).
        for _ in 0..n {
            items.push(self.value_in(elem, iv).0);
        }
        let room = if mutable { self.rng.below(9) } else { 0 };
        let name = self.name("xs");
        let list = ListInfo { lo: n, cap: n + room, len_now: n, mutable, decl_mult: self.mult, frozen: 0, walk_pos: None, lent_mut: false };
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: elem, iv, assignable: false, list: Some(list), item: false, data: false, unseen: false, en: None, simd: None });
        Stmt::ListLet { name, mutable, elem, items }
    }

    /// A loop over every position of a list whose trips are independent,
    /// which CCA may run in slices on several threads (34d9c18): each item
    /// rewritten from itself, or all of them added into one sum printed
    /// after. Either way the answer is the in-order one.
    fn list_loop(&mut self) -> Stmt {
        let ls = self.lists();
        let v = self.rng.pick(&ls).clone();
        let l = v.list.clone().unwrap();
        let i = self.name("i");
        let at = || Box::new(Expr::At(v.name.clone(), Box::new(Expr::Var(i.clone()))));
        let bounds = |body: Vec<Stmt>| Stmt::For { perm: false, cheat: 0, name: i.clone(), ty: Ty::U64, start: Expr::Lit(1), stop: Expr::Len(v.name.clone()), step: None, body };
        self.positioned = true;
        if l.mutable && l.frozen == 0 && self.rng.chance(35) {
            // the same through the item: for 'x' in lend.mut 'xs' { change 'x' = … }
            let x = self.name("x");
            let (plus, piv) = (Expr::Bin(BinOp::Add, Box::new(Expr::To(v.ty, Box::new(Expr::Var(x.clone())))), Box::new(Expr::Lit(1))), Iv::new(v.iv.lo + 1, v.iv.hi + 1));
            let (value, _) = self.fit(plus, piv, v.iv, v.ty);
            let set = Stmt::Assign { tier: Tier::Default, name: x.clone(), value };
            let mut body = vec![set];
            // after the item's last use, the list may be read, and shows the
            // change already made (owner: a, 257bde8)
            match self.rng.below(3) {
                0 => body.push(Stmt::Print { signed: false, value: Expr::Len(v.name.clone()) }),
                1 => {
                    let t = if v.ty.signed() { Ty::I64 } else { Ty::U64 };
                    body.push(Stmt::Print { signed: v.ty.signed(), value: Expr::To(t, Box::new(Expr::At(v.name.clone(), Box::new(Expr::Lit(1))))) });
                }
                _ => {}
            }
            return Stmt::ForEach { pos: None, item: x, list: v.name.clone(), body, mutable: true };
        }
        if l.mutable && l.frozen == 0 && self.rng.chance(50) {
            // 'xs'['i'] = ('xs'['i'] + 1), folded back into what the items promise
            let (plus, piv) = (Expr::Bin(BinOp::Add, at(), Box::new(Expr::Lit(1))), Iv::new(v.iv.lo + 1, v.iv.hi + 1));
            let (value, _) = self.fit(plus, piv, v.iv, v.ty);
            let set = Stmt::SetAt { tier: Tier::Default, name: v.name.clone(), pos: Expr::Var(i.clone()), value };
            return bounds(vec![set]);
        }
        let acc = self.name("v");
        let init = Stmt::Let { tier: Tier::Default, name: acc.clone(), mutability: Mutability::Mut, ty: Ty::I64.into(), value: Expr::Lit(0) };
        let item = Expr::To(Ty::I64, Box::new(Expr::Bin(BinOp::Mod, at(), Box::new(Expr::Lit(100)))));
        let add = Stmt::Assign { tier: Tier::Default, name: acc.clone(), value: Expr::Bin(BinOp::Add, Box::new(Expr::Var(acc.clone())), Box::new(item)) };
        Stmt::Block { tier: Tier::Default, body: vec![init, bounds(vec![add]), Stmt::Print { signed: true, value: Expr::Var(acc) }] }
    }

    /// `list.fill[v, n]` (owner, 2026-09-30): a literal count, or one
    /// computed and kept in 0..8 — or, when a risk is taken, one that may
    /// be below 0, so the count's check fires.
    fn list_fill(&mut self) -> Stmt {
        let elem = self.int_ty();
        let mutable = self.rng.chance(60);
        let iv = self.ghost(elem);
        self.risky = false;
        let (value, _) = self.value_in(elem, iv);
        let (count, civ) = if self.rng.chance(40) {
            let n = self.rng.range(0, 8);
            (Expr::Lit(n), Iv::point(n))
        } else {
            let t = *self.rng.pick(&[Ty::I32, Ty::I64, Ty::U8, Ty::U64]);
            let target = if t.signed() && self.take_risk() { Iv::new(-3, 8) } else { Iv::new(0, 8) };
            self.value_in(t, target)
        };
        let (lo, cap) = (civ.lo.max(0) as u64, civ.hi.max(0) as u64);
        let room = if mutable { self.rng.below(5) } else { 0 };
        let name = self.name("xs");
        let list = ListInfo { lo, cap: cap + room, len_now: cap, mutable, decl_mult: self.mult, frozen: 0, walk_pos: None, lent_mut: false };
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: elem, iv, assignable: false, list: Some(list), item: false, data: false, unseen: false, en: None, simd: None });
        Stmt::ListFill { name, mutable, elem, value, count }
    }

    /// A position in `v`: guarded unless the generator takes a risk.
    fn position(&mut self, v: &VarInfo) -> Expr {
        self.positioned = true;
        let l = v.list.clone().unwrap();
        if self.take_risk() {
            // Unguarded: may be 0 or past the end, so the check can fire.
            let top = l.cap as i128 + 2;
            return if self.rng.chance(50) { Expr::Lit(self.rng.range(0, top)) } else { self.value_in(Ty::U64, Iv::new(0, top)).0 };
        }
        if let Some(p) = &l.walk_pos {
            if l.lo == 0 || self.rng.chance(40) {
                return Expr::Var(p.clone());
            }
        }
        if self.rng.chance(35) {
            return Expr::Lit(self.rng.range(1, l.lo as i128));
        }
        // 1 + (e mod list.length[lend 'xs']): in 1..length whatever e is.
        let (e, _) = self.value_in(Ty::U64, Ty::U64.range());
        let m = Expr::Bin(BinOp::Mod, Box::new(e), Box::new(Expr::Len(v.name.clone())));
        if self.rng.chance(50) {
            Expr::Bin(BinOp::Add, Box::new(Expr::Lit(1)), Box::new(m))
        } else {
            Expr::Bin(BinOp::Add, Box::new(m), Box::new(Expr::Lit(1)))
        }
    }

    fn set_at(&mut self) -> Stmt {
        let v = self.rng.pick(&self.settable()).clone();
        self.risky = false;
        self.positioned = false;
        let pos = self.position(&v);
        let (value, _) = self.value_in(v.ty, v.iv);
        self.positioned |= self.uses_unseen(&value) || self.uses_unseen(&pos);
        let tier = self.tier(self.risky);
        Stmt::SetAt { tier, name: v.name, pos, value }
    }

    fn append(&mut self) -> Stmt {
        let v = self.rng.pick(&self.appendable()).clone();
        let mult = self.mult;
        let l = self.list_mut(&v.name);
        l.len_now += (mult / l.decl_mult.max(1)).max(1);
        // The value takes the item type as its context, as any argument
        // takes its parameter's (owner, 1967d08).
        let (value, _) = self.value_in(v.ty, v.iv);
        Stmt::Append { name: v.name, value }
    }

    fn for_each(&mut self, depth: u32) -> Stmt {
        let walkable = self.walkable();
        let changeable: Vec<VarInfo> = walkable.iter().filter(|v| v.list.as_ref().map_or(false, |l| l.mutable && l.frozen == 0)).cloned().collect();
        let v = if !changeable.is_empty() && self.rng.chance(60) { self.rng.pick(&changeable).clone() } else { self.rng.pick(&walkable).clone() };
        let cap = v.list.as_ref().unwrap().cap;
        // `for 'x' in lend.mut 'xs'`: the item is changed and written back
        // (owner, 2026-09-30); only over a list that may change and is not
        // being walked already.
        let l = v.list.as_ref().unwrap();
        let mutable = l.mutable && l.frozen == 0 && self.rng.chance(60);
        let item = self.name("x");
        let pos = self.rng.chance(50).then(|| self.name("i"));
        let saved = (self.mult, self.in_loop);
        self.mult = self.mult.saturating_mul(cap.max(1));
        self.in_loop = LoopKind::Loop;
        let old_pos = {
            let l = self.list_mut(&v.name);
            l.frozen += 1;
            l.lent_mut = mutable;
            std::mem::replace(&mut l.walk_pos, pos.clone())
        };
        self.scopes.push(vec![]);
        self.declare(item.clone(), v.ty, v.iv, mutable);
        self.scopes.last_mut().unwrap().last_mut().unwrap().item = true;
        self.mark_unseen(&item);
        if let Some(p) = &pos {
            self.declare(p.clone(), Ty::U64, Iv::new(1, cap.max(1) as i128), false);
            self.mark_unseen(p);
        }
        let nb = self.stmt_count() / 2 + 1;
        let mut body = vec![];
        if mutable {
            self.risky = false;
            self.positioned = true;
            let (value, _) = self.value_in(v.ty, v.iv);
            body.push(Stmt::Assign { tier: Tier::Default, name: item.clone(), value });
        }
        body.extend(self.stmts(nb, depth - 1));
        self.scopes.pop();
        {
            let l = self.list_mut(&v.name);
            l.frozen -= 1;
            l.lent_mut = false;
            l.walk_pos = old_pos;
        }
        (self.mult, self.in_loop) = saved;
        Stmt::ForEach { pos, item, list: v.name, body, mutable }
    }

    // ------------------------------------------------------------ borrows

    /// `return lend 'p';` or `return b[lend …];`, and what it can borrow.
    fn borrow_return(&mut self, index: usize, params: &[String]) -> (Expr, Vec<usize>) {
        let callees: Vec<BSig> = self.bsigs[..index].iter().filter(|b| !b.wrong).cloned().collect();
        if !callees.is_empty() && self.rng.chance(50) {
            let c = self.rng.pick(&callees).clone();
            let map: Vec<usize> = (0..c.arity).map(|_| self.rng.below(params.len() as u64) as usize).collect();
            let args = map.iter().map(|&m| Expr::Lend(params[m].clone())).collect();
            let mut deps: Vec<usize> = c.deps.iter().map(|&d| map[d]).collect();
            deps.sort();
            deps.dedup();
            return (Expr::Call(c.name, args), deps);
        }
        let k = self.rng.below(params.len() as u64) as usize;
        (Expr::Lend(params[k].clone()), vec![k])
    }

    fn borrow_fn(&mut self, index: usize, wrong: bool) -> BFunc {
        let name = format!("b{}", index + 1);
        let n = 1 + self.rng.below(3) as usize;
        let params: Vec<String> = ["a", "b", "c"][..n].iter().map(|s| s.to_string()).collect();
        let len = |p: &str| Box::new(Expr::Len(p.to_string()));
        let mut body = vec![];
        let mut deps = vec![];
        if self.rng.chance(35) {
            // Two ways out, chosen by the lists' lengths.
            let a = self.rng.pick(&params).clone();
            let cond = if self.rng.chance(50) {
                let b = self.rng.pick(&params).clone();
                Expr::Cmp(CmpOp::Ge, len(&a), len(&b))
            } else {
                Expr::Cmp(CmpOp::Gt, len(&a), Box::new(Expr::Lit(self.rng.range(0, 3))))
            };
            let (e, d) = self.borrow_return(index, &params);
            body.push(Stmt::If { arms: vec![(cond, vec![Stmt::Return(e)])], otherwise: None });
            deps.extend(d);
        }
        let (e, d) = self.borrow_return(index, &params);
        body.push(Stmt::Return(e));
        deps.extend(d);
        deps.sort();
        deps.dedup();
        // The signature: no groups (Precog reads the body), or @x on exactly
        // what the body can return and @y (or nothing) on the rest.
        // A strict file names the group (owner, 09044b6).
        let annotated = self.strict || self.rng.chance(65);
        let mut groups: Vec<Option<String>> = (0..n)
            .map(|i| if !annotated { None } else if deps.contains(&i) { Some("x".into()) } else if self.rng.chance(70) { Some("y".into()) } else { None })
            .collect();
        let mut ret_group = annotated.then(|| "x".to_string());
        let mut unsaid = false;
        if wrong {
            match self.rng.below(3) {
                0 => {
                    // A borrow of a local, which dies at the return.
                    let loc = "loc".to_string();
                    let (v, _) = self.literal(self.belem);
                    body = vec![Stmt::ListLet { name: loc.clone(), mutable: false, elem: self.belem, items: vec![v] }, Stmt::Return(Expr::Lend(loc))];
                }
                1 => {
                    // One returned parameter left out of the result's group.
                    let d = *self.rng.pick(&deps);
                    for (i, g) in groups.iter_mut().enumerate() {
                        *g = if i == d { Some("y".into()) } else if deps.contains(&i) { Some("x".into()) } else { g.clone() };
                    }
                    ret_group = Some("x".into());
                }
                _ => unsaid = true,
            }
        }
        self.bsigs.push(BSig { name: name.clone(), arity: n, deps, wrong });
        BFunc { name, params: params.into_iter().zip(groups).collect(), elem: self.belem, ret_group, unsaid, body }
    }

    /// Print a use of a borrowed list: its first item or its length.
    fn use_borrow(&mut self, r: &str) -> Stmt {
        // A position in a borrowed list is proven only from its length,
        // which a strict file cannot see from here.
        if self.strict || self.rng.chance(40) {
            return Stmt::Print { signed: false, value: Expr::Len(r.to_string()) };
        }
        let at = Expr::At(r.to_string(), Box::new(Expr::Lit(1)));
        if self.belem.signed() {
            Stmt::Print { signed: true, value: Expr::To(Ty::I64, Box::new(at)) }
        } else {
            Stmt::Print { signed: false, value: Expr::To(Ty::U64, Box::new(at)) }
        }
    }

    /// A block of lists, borrows of them handed back through the borrow
    /// functions (some through earlier borrows), uses, and maybe a give, an
    /// append or a change of one list. `mistake`: Some(Some(i)) calls the
    /// wrong function b(i+1); Some(None) changes a list while a borrow of it
    /// is used after; None is right by construction.
    fn episode(&mut self, mistake: Option<Option<usize>>) -> Stmt {
        let elem = self.belem;
        let mut body = vec![];
        let nown = if matches!(mistake, Some(Some(_))) { 3 } else { 1 + self.rng.below(3) as usize };
        let mut owners = vec![];
        for _ in 0..nown {
            let o = self.name("o");
            let mutable = self.rng.chance(60) || mistake == Some(None);
            let k = 1 + self.rng.below(3);
            let items = (0..k).map(|_| self.literal(elem).0).collect();
            body.push(Stmt::ListLet { name: o.clone(), mutable, elem, items });
            owners.push((o, mutable));
        }
        // `m1[lend.mut 'oA', lend 'oB']`: two lists, before any borrow of
        // them exists — or, as the mistake, one list lent both ways. Not in
        // a `check`: the length `m1` leaves is a callee's fact the reads
        // below would lean on (seed 11647).
        if self.mutator && self.in_check == 0 {
            if mistake == Some(None) && self.rng.chance(40) {
                let o = owners[0].0.clone();
                body.push(Stmt::Src(format!("m1[lend.mut '{o}', lend '{o}'];")));
                body.push(Stmt::Print { signed: false, value: Expr::Len(o) });
                self.unevaluable = true;
                return Stmt::Block { tier: Tier::Default, body };
            }
            if mistake.is_none() && nown >= 2 && owners[0].1 && self.rng.chance(30) {
                body.push(Stmt::Src(format!("m1[lend.mut '{}', lend '{}'];", owners[0].0, owners[1].0)));
                self.unevaluable = true;
            }
        }
        // Each borrow: its name and the owners it may borrow.
        let mut borrows: Vec<(String, Vec<usize>)> = vec![];
        let usable: Vec<BSig> = match mistake {
            Some(Some(bad)) => vec![self.bsigs[bad].clone()],
            _ => self.bsigs.iter().filter(|b| !b.wrong).cloned().collect(),
        };
        let nb = if matches!(mistake, Some(Some(_))) { 1 } else { 1 + self.rng.below(3) as usize };
        for _ in 0..nb {
            let f = self.rng.pick(&usable).clone();
            let mut args = vec![];
            let mut arg_deps = vec![];
            for _ in 0..f.arity {
                if matches!(mistake, Some(Some(_))) {
                    // Distinct lists, so the mistake shows even to a check
                    // made when the call runs (seed 3143, 2026-09-29).
                    let i = args.len() % nown;
                    args.push(Expr::Lend(owners[i].0.clone()));
                    arg_deps.push(vec![i]);
                } else if !borrows.is_empty() && self.rng.chance(35) {
                    let (r, d) = self.rng.pick(&borrows).clone();
                    args.push(Expr::Lend(r));
                    arg_deps.push(d);
                } else {
                    let i = self.rng.below(nown as u64) as usize;
                    args.push(Expr::Lend(owners[i].0.clone()));
                    arg_deps.push(vec![i]);
                }
            }
            let mut deps: Vec<usize> = f.deps.iter().flat_map(|&d| arg_deps[d].clone()).collect();
            deps.sort();
            deps.dedup();
            let r = self.name("r");
            body.push(Stmt::Borrow { name: r.clone(), value: Expr::Call(f.name.clone(), args) });
            borrows.push((r, deps));
        }
        for _ in 0..self.rng.below(3) {
            let r = self.rng.pick(&borrows).0.clone();
            body.push(self.use_borrow(&r));
        }
        let kill = mistake == Some(None) || (mistake.is_none() && self.rng.chance(70));
        let mut after: Vec<String> = borrows.iter().map(|b| b.0.clone()).collect();
        if kill {
            // The owner to give away or change; for a mistake, one some borrow depends on.
            let pick: Vec<usize> = if mistake == Some(None) {
                (0..nown).filter(|i| borrows.iter().any(|b| b.1.contains(i))).collect()
            } else {
                (0..nown).collect()
            };
            if let Some(&o) = (!pick.is_empty()).then(|| self.rng.pick(&pick)) {
                let (oname, mutable) = owners[o].clone();
                let kinds = if mutable { 3 } else { 1 };
                body.push(match self.rng.below(kinds) {
                    0 => Stmt::Give { name: self.name("g"), from: oname.clone() },
                    1 => Stmt::Append { name: oname.clone(), value: self.literal(elem).0 },
                    _ => Stmt::SetAt { tier: Tier::Default, name: oname.clone(), pos: Expr::Lit(1), value: self.literal(elem).0 },
                });
                let dependent: Vec<String> = borrows.iter().filter(|b| b.1.contains(&o)).map(|b| b.0.clone()).collect();
                if mistake == Some(None) {
                    after = vec![self.rng.pick(&dependent).clone()];
                } else {
                    // Right by construction: only borrows that do not need it are used after.
                    after.retain(|r| !dependent.contains(r));
                }
            }
        }
        let nafter = if mistake == Some(None) { 1 } else { self.rng.below(3) as usize };
        if !after.is_empty() {
            for _ in 0..nafter {
                let r = self.rng.pick(&after).clone();
                body.push(self.use_borrow(&r));
            }
        }
        Stmt::Block { tier: Tier::Default, body }
    }

    // ------------------------------------------------------------ enums

    fn enum_def(&mut self, k: usize) {
        let n = 2 + self.rng.below(3) as usize;
        let mut cases = vec![];
        let mut ivs = vec![];
        // A `layout.C enum` (owner, 2026-09-27): a distinct number per case, no data.
        let c_enum = self.rng.chance(30);
        let mut values: Vec<i128> = vec![];
        while c_enum && values.len() < n {
            let v = if self.rng.chance(20) { self.rng.range(-2147483648, 2147483647) } else { self.rng.range(-300, 300) };
            if !values.contains(&v) {
                values.push(v);
            }
        }
        for i in 0..n {
            let (t, iv) = if !c_enum && self.rng.chance(55) {
                let t = self.int_ty();
                (Some(t), Some(self.ghost(t)))
            } else {
                (None, None)
            };
            cases.push((format!("c{}", i + 1), t));
            ivs.push(iv);
        }
        self.enums.push(EnumGen { def: EnumDef { name: format!("E{}", k + 1), cases, values: c_enum.then_some(values) }, ivs });
    }

    /// `E1.c2[v]` or `E1.c1`, its data within the case's promise.
    /// `whole`: the value is a declaration's or a change's whole value, the
    /// only place `… otherwise { … }` may stand.
    fn case_value(&mut self, ei: usize, whole: bool) -> Expr {
        let e = self.enums[ei].clone();
        let c = self.rng.below(e.def.cases.len() as u64) as usize;
        if let Some(vals) = &e.def.values {
            if whole && self.rng.chance(50) {
                // A number into the C enum: mostly one a case has, sometimes
                // one next to it, so the fallback runs.
                let w = vals[c] + if self.rng.chance(25) { 1 } else { 0 };
                let t = if Ty::I32.range().has(w) && self.rng.chance(60) { Ty::I32 } else { Ty::I64 };
                let (x, _) = if self.rng.chance(40) { (Expr::Lit(w), Iv::point(w)) } else { self.value_in(t, Iv::point(w)) };
                let fb = self.rng.below(vals.len() as u64) as usize;
                let fallback = Expr::Case(e.def.name.clone(), e.def.cases[fb].0.clone(), None);
                return Expr::ToEnum(e.def.name.clone(), Box::new(x), Box::new(fallback));
            }
        }
        let data = match (e.def.cases[c].1, e.ivs[c]) {
            (Some(t), Some(iv)) => Some(Box::new(self.value_in(t, iv).0)),
            _ => None,
        };
        Expr::Case(e.def.name.clone(), e.def.cases[c].0.clone(), data)
    }

    fn enum_let(&mut self) -> Stmt {
        let ei = self.rng.below(self.enums.len() as u64) as usize;
        let mutable = self.rng.chance(40);
        let value = self.case_value(ei, true);
        let name = self.name("e");
        let en = self.enums[ei].def.name.clone();
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: Ty::I64, iv: Iv::point(0), assignable: mutable, list: None, item: false, data: false, unseen: false, en: Some(ei), simd: None });
        Stmt::EnumLet { name, mutable, en, value }
    }

    fn enum_assign(&mut self) -> Stmt {
        let vs: Vec<VarInfo> = self.enum_vars().into_iter().filter(|v| v.assignable).collect();
        let v = self.rng.pick(&vs).clone();
        let value = self.case_value(v.en.unwrap(), true);
        Stmt::Assign { tier: Tier::Default, name: v.name, value }
    }

    /// What to match on: an enum name in scope, or a case built right there.
    fn scrutinee(&mut self) -> (Expr, usize) {
        let evs = self.enum_vars();
        if !evs.is_empty() && self.rng.chance(70) {
            let v = self.rng.pick(&evs).clone();
            return (Expr::Var(v.name), v.en.unwrap());
        }
        let ei = self.rng.below(self.enums.len() as u64) as usize;
        (self.case_value(ei, false), ei)
    }

    /// Open an arm's scope, with its data's name if the case has data.
    fn open_arm(&mut self, ei: usize, c: usize) -> Option<String> {
        self.scopes.push(vec![]);
        let e = self.enums[ei].clone();
        let (t, iv) = (e.def.cases[c].1?, e.ivs[c]?);
        let b = self.name("d");
        self.declare(b.clone(), t, iv, false);
        self.scopes.last_mut().unwrap().last_mut().unwrap().data = true;
        // Only the generator knows the data's range.
        self.mark_unseen(&b);
        Some(b)
    }

    /// `match … { … }` giving a value of type `t` within `target`; now and
    /// then an arm leaves instead (`return`, `break`, `continue`).
    fn match_value(&mut self, t: Ty, target: Iv) -> Expr {
        let (scrut, ei) = self.scrutinee();
        let mut arms = vec![];
        for c in 0..self.enums[ei].def.cases.len() {
            let bind = self.open_arm(ei, c);
            let leave = self.rng.chance(12) && (self.in_fn.is_some() || self.in_loop != LoopKind::None);
            let arm = if leave {
                let s = match self.in_fn {
                    Some((rt, riv)) if self.in_loop == LoopKind::None || self.rng.chance(50) => {
                        Stmt::Return(if rt == Ty::Bool { self.bool_expr(1) } else { self.value_in(rt, riv).0 })
                    }
                    _ => if self.rng.chance(50) { Stmt::Break } else { Stmt::Continue },
                };
                Arm { case: self.enums[ei].def.cases[c].0.clone(), bind, value: None, body: vec![s] }
            } else {
                let (v, _) = self.value_in(t, target);
                Arm { case: self.enums[ei].def.cases[c].0.clone(), bind, value: Some(v), body: vec![] }
            };
            self.scopes.pop();
            arms.push(arm);
        }
        Expr::Match(Box::new(scrut), arms)
    }

    fn match_stmt(&mut self, depth: u32) -> Stmt {
        let (scrut, ei) = self.scrutinee();
        let mut arms = vec![];
        for c in 0..self.enums[ei].def.cases.len() {
            let bind = self.open_arm(ei, c);
            let n = if self.rng.chance(25) { 0 } else { self.stmt_count() / 3 + 1 };
            // A match statement may be an arm's only statement: the outer
            // match decides that it is a statement (owner, a4bdac1).
            let body = self.stmts(n, depth - 1);
            self.scopes.pop();
            arms.push(Arm { case: self.enums[ei].def.cases[c].0.clone(), bind, value: None, body });
        }
        Stmt::MatchStmt { scrut, arms }
    }

    // ------------------------------------------------------------ simd

    /// The target's shapes (ARM64's 128 bits, the oracle's target).
    fn lanes(t: Ty) -> usize {
        128 / t.bits() as usize
    }

    /// A vector operand of lane type `t`: a name of that type, or a fill.
    /// Its lanes' interval.
    fn simd_operand(&mut self, t: Ty, _hint: Iv) -> (Expr, Iv) {
        let vs = self.simd_vars(Some(t));
        if !vs.is_empty() && self.rng.chance(65) {
            let v = self.rng.pick(&vs).clone();
            return (Expr::Var(v.name), v.iv);
        }
        let (e, iv) = self.literal(t);
        (Expr::SimdFill(Box::new(e)), iv)
    }

    /// A vector of lane type `t` whose every lane stays in `target`: a fill,
    /// or one lane-wise operator on two operands, switched to a wrapping or
    /// bitwise one where a lane could leave `t`, since each lane operation
    /// is checked at the lane's type (owner: b, fc1cea9) — unless a risk is
    /// taken.
    fn simd_value(&mut self, t: Ty, target: Iv) -> (Expr, Iv) {
        let r = t.range();
        if self.simd_vars(Some(t)).is_empty() || self.rng.chance(35) {
            let (e, iv) = self.value_in(t, target);
            return (Expr::SimdFill(Box::new(e)), iv);
        }
        let (a, ia) = self.simd_operand(t, target);
        use BinOp::*;
        let op = *self.rng.pick(&[Add, Sub, Mul, Div, Mod, AndBit, OrBit, Xor, Shl, Shr, WAdd, WSub, WMul]);
        let op = if !self.cfg.simd_wrap && matches!(op, WAdd | WSub | WMul) { Xor } else { op };
        let (b, ib, op) = match op {
            Div | Mod => {
                // a nonzero divisor, never -1 beside the minimum
                let k = if t.signed() && self.rng.chance(30) { -self.rng.range(2, 9) } else { self.rng.range(1, 9) };
                (Expr::SimdFill(Box::new(Expr::Lit(k))), Iv::point(k), op)
            }
            Shl | Shr => {
                let k = self.rng.range(0, t.bits() as i128 - 1);
                (Expr::SimdFill(Box::new(Expr::Lit(k))), Iv::point(k), op)
            }
            _ => {
                let (b, ib) = self.simd_operand(t, target);
                (b, ib, op)
            }
        };
        let out = match op {
            Add => corners(ia, ib, |x, y| Some(x + y)),
            Sub => corners(ia, ib, |x, y| Some(x - y)),
            Mul => corners(ia, ib, |x, y| x.checked_mul(y)),
            Div => corners(ia, ib, |x, y| Some(floor_div(x, y))),
            Mod => Some(if ib.lo > 0 { Iv::new(0, ib.hi - 1) } else { Iv::new(ib.lo + 1, 0) }),
            Shl => corners(ia, ib, |x, k| x.checked_mul(1i128 << k)),
            Shr if ia.lo >= 0 => Some(Iv::new(ia.lo >> ib.hi, ia.hi >> ib.lo)),
            AndBit | OrBit | Xor if ia.lo >= 0 && ib.lo >= 0 => {
                let top = ia.hi.max(ib.hi);
                let mask = if top == 0 { 0 } else { (1i128 << (128 - top.leading_zeros())) - 1 };
                Some(Iv::new(0, if op == AndBit { ia.hi.min(ib.hi) } else { mask }))
            }
            _ => Some(r),
        };
        let (op, iv) = match out {
            Some(iv) if iv.within(r) => (op, iv),
            // Left to overflow a lane on purpose: each lane operation is
            // checked at the lane type (owner: b, fc1cea9).
            _ if matches!(op, Add | Sub | Mul) && self.take_risk() => (op, r),
            // a lane could leave the type: wrap instead
            _ if self.cfg.simd_wrap => (*self.rng.pick(&[WAdd, WSub, WMul, Xor]), r),
            _ => (*self.rng.pick(&[AndBit, OrBit, Xor]), r),
        };
        (Expr::Bin(op, Box::new(a), Box::new(b)), iv)
    }

    fn simd_let(&mut self) -> Stmt {
        let t = self.int_ty();
        let n = Self::lanes(t);
        let mutable = self.rng.chance(35);
        // A vector that changes promises only its type.
        let target = if mutable { t.range() } else { self.ghost(t) };
        let (value, iv) = self.simd_value(t, target);
        let name = self.name("s");
        let promise = if mutable { t.range() } else { iv };
        self.scopes.last_mut().unwrap().push(VarInfo { name: name.clone(), ty: t, iv: promise, assignable: mutable, list: None, item: false, data: false, unseen: false, en: None, simd: Some(n) });
        Stmt::SimdLet { name, mutable, elem: t, lanes: n, value }
    }

    fn simd_assign(&mut self) -> Stmt {
        let vs: Vec<VarInfo> = self.simd_vars(None).into_iter().filter(|v| v.assignable).collect();
        let v = self.rng.pick(&vs).clone();
        let (value, _) = self.simd_value(v.ty, v.ty.range());
        Stmt::Assign { tier: Tier::Default, name: v.name, value }
    }

    /// `change 's'[k] = v;` (owner, 8d8a302): a literal lane, or rarely one
    /// past the last when a risk is taken.
    fn lane_write(&mut self) -> Stmt {
        let vs: Vec<VarInfo> = self.simd_vars(None).into_iter().filter(|v| v.assignable).collect();
        let v = self.rng.pick(&vs).clone();
        let n = v.simd.unwrap() as i128;
        self.risky = false;
        self.positioned = true;
        let k = if self.take_risk() { n + 1 } else { self.rng.range(1, n) };
        let (value, _) = self.value_in(v.ty, v.iv);
        Stmt::SetAt { tier: Tier::Default, name: v.name, pos: Expr::Lit(k), value }
    }

    /// One number from a vector of lane type `t`: a fold, or one lane.
    fn simd_read(&mut self, v: VarInfo, t: Ty) -> (Expr, Iv) {
        let n = v.simd.unwrap() as i128;
        let var = Box::new(Expr::Var(v.name.clone()));
        let sum = Iv::new(v.iv.lo * n, v.iv.hi * n);
        match self.rng.below(4) {
            0 if self.cfg.simd_lanes => {
                let k = self.rng.range(1, n);
                (Expr::At(v.name, Box::new(Expr::Lit(k))), v.iv)
            }
            1 if sum.within(t.range()) || self.take_risk() => (Expr::SimdFold("sum", var), sum.meet(t.range()).unwrap_or(t.range())),
            2 => (Expr::SimdFold("min", var), v.iv),
            _ => (Expr::SimdFold("max", var), v.iv),
        }
    }

    // ------------------------------------------------------------ deci

    /// A deci literal of at most `max` significant digits, sometimes with
    /// trailing zeros (kept, as IEEE keeps them).
    fn deci_lit(&mut self, max: usize) -> String {
        let n = 1 + self.rng.below(max.min(6) as u64) as usize;
        let mut digits: String = (0..n).map(|i| {
            let d = self.rng.below(10) as u8;
            (b'0' + if i == 0 && d == 0 { 1 } else { d }) as char
        }).collect();
        if n >= 2 && self.rng.chance(25) {
            digits.replace_range(n - 1.., "0");
        }
        let point = self.rng.below(n as u64 + 1) as usize;
        let mut t = if point == 0 { format!("0.{digits}") } else if point == n { format!("{digits}.0") } else { format!("{}.{}", &digits[..point], &digits[point..]) };
        // `0.xyz` has a leading zero that is not a digit of the value, but
        // stay clear of the edge: keep it within `max` characters of digits.
        if t.chars().filter(|c| c.is_ascii_digit()).count() > max {
            t = digits[..1].to_string() + ".0";
        }
        if self.rng.chance(20) { format!("-{t}") } else { t }
    }

    /// Text to read as a number: well-formed, badly formed, too long, too big.
    fn number_text(&mut self) -> String {
        const POOL: &[&str] = &["42", "-7", "0", "1.250", "-0.5", "2.5e3", "1e400", "abc", "", "12x", " 5", "+3", "3.", ".5", "0.12345678901234567", "99999999999", "-1", "1e-3", "inf", "nan", "00012", "1_000", "7.0"];
        if self.rng.chance(70) {
            POOL[self.rng.below(POOL.len() as u64) as usize].to_string()
        } else {
            let v = self.rng.range(-100000, 100000);
            if self.rng.chance(50) { v.to_string() } else { format!("{}.{}", v, self.rng.below(1000)) }
        }
    }

    /// A block of deci work: values from literals and from each other,
    /// conversions from integers, text, bins and other widths, and what is
    /// seen of them as integers (scaled `to.int64[…] otherwise -1`, a
    /// comparison, the length of its text), plus text read as integers.
    fn deci_episode(&mut self) -> Stmt {
        self.unevaluable = true;
        let w = *self.rng.pick(&[32u32, 64, 64, 128]);
        let ty = format!("deci{w}");
        let digits = match w { 32 => 7, 64 => 16, _ => 34 };
        let mut body = vec![];
        let mut decis: Vec<String> = vec![];
        let first = self.name("d");
        body.push(Stmt::TypedLet { name: first.clone(), mutable: false, ty: ty.clone(), value: Expr::Num(self.deci_lit(digits)) });
        decis.push(first);
        for _ in 0..1 + self.rng.below(4) {
            let a = Expr::Var(self.rng.pick(&decis).clone());
            let value = match self.rng.below(9) {
                0..=4 => {
                    let op = self.fraction_op(true);
                    let b = if self.rng.chance(50) { Expr::Var(self.rng.pick(&decis).clone()) } else { Expr::Num(self.deci_lit(digits)) };
                    // a divisor of 0 stops both engines alike; keep it rare
                    let b = match (&b, op.division()) {
                        (Expr::Num(t), Some(_)) if t.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') => Expr::Num("1.5".into()),
                        (Expr::Var(_), Some(_)) if !self.take_risk() => Expr::Num("2.5".into()),
                        _ => b,
                    };
                    Expr::Bin(op, Box::new(a), Box::new(b))
                }
                // a small exponent: values of at most 6 digits stay far inside
                // even deci32's range, since this may sit in `check` code
                5 => Expr::Bin(BinOp::Pow, Box::new(a), Box::new(Expr::Lit(self.rng.range(-3, 4)))),
                6 => Expr::Neg(Box::new(a)),
                7 => {
                    // from an integer in scope: `otherwise` only when its type
                    // could have more digits than the deci holds
                    let ints: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
                    if ints.is_empty() {
                        Expr::Num(self.deci_lit(digits))
                    } else {
                        let v = self.rng.pick(&ints).clone();
                        let most = match v.ty { Ty::I8 | Ty::U8 => 3, Ty::I16 | Ty::U16 => 5, Ty::I32 | Ty::U32 => 10, Ty::I64 => 19, _ => 20 };
                        let fb = (most > digits).then(|| Box::new(Expr::Num(self.deci_lit(digits))));
                        Expr::Conv(ty.clone(), Box::new(Expr::Var(v.name)), fb)
                    }
                }
                _ => Expr::Conv(ty.clone(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Num(self.deci_lit(digits))))),
            };
            let name = self.name("d");
            let typed = !matches!(value, Expr::Conv(..));
            body.push(if typed { Stmt::TypedLet { name: name.clone(), mutable: false, ty: ty.clone(), value } } else { Stmt::AutoLet { name: name.clone(), value } });
            decis.push(name);
        }
        // between widths, and from a bin
        if self.rng.chance(40) {
            let (to, to_digits) = if w == 32 { ("deci64", 16) } else { ("deci32", 7) };
            let src = Expr::Var(self.rng.pick(&decis).clone());
            let name = self.name("d");
            let fb = Box::new(Expr::Num(self.deci_lit(to_digits)));
            body.push(Stmt::AutoLet { name: name.clone(), value: Expr::Conv(to.into(), Box::new(src), (to == "deci32").then_some(fb)) });
            let z = self.name("z");
            body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("int64".into(), Box::new(Expr::Bin(BinOp::Mul, Box::new(Expr::Var(name)), Box::new(Expr::Num("1000".into())))), Some(Box::new(Expr::Lit(-1)))) });
            body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
        }
        if self.rng.chance(35) {
            let b = self.name("b");
            let bin = *self.rng.pick(&["0.1", "0.2", "2.5", "-1.75", "3.0", "0.001", "123.456"]);
            body.push(Stmt::TypedLet { name: b.clone(), mutable: false, ty: "bin64".into(), value: Expr::Num(bin.into()) });
            // a bin into a deci says which form (owner, 2026-10-02)
            let conv = if self.cfg.deci_useful && self.rng.chance(50) { format!("useful.{ty}") } else { format!("stored.{ty}") };
            let name = self.name("d");
            body.push(Stmt::AutoLet { name: name.clone(), value: Expr::Conv(conv, Box::new(Expr::Var(b)), None) });
            decis.push(name);
        }
        // a range type, sometimes pushed outside by a risk taken
        if w == 64 && self.rng.chance(30) {
            let name = self.name("d");
            let inside = format!("0.{}", self.rng.range(1, 99));
            let value = if self.take_risk() { Expr::Bin(BinOp::Mul, Box::new(Expr::Var(self.rng.pick(&decis).clone())), Box::new(Expr::Num("3.0".into()))) } else { Expr::Num(inside) };
            body.push(Stmt::TypedLet { name: name.clone(), mutable: false, ty: "deci64[0.0, 1.0]".into(), value });
            decis.push(name);
        }
        // what is seen: scaled into an integer, compared, and as text
        for _ in 0..1 + self.rng.below(3) {
            let d = Expr::Var(self.rng.pick(&decis).clone());
            match self.rng.below(3) {
                0 => {
                    let scale = *self.rng.pick(&["1", "10", "100", "1000", "1000000"]);
                    let z = self.name("z");
                    body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("int64".into(), Box::new(Expr::Bin(BinOp::Mul, Box::new(d), Box::new(Expr::Num(scale.into())))), Some(Box::new(Expr::Lit(-1)))) });
                    body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
                }
                1 => {
                    let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Lt, CmpOp::Ge]);
                    let c = Expr::Cmp(op, Box::new(d), Box::new(Expr::Num(self.deci_lit(digits))));
                    body.push(Stmt::If { arms: vec![(c, vec![Stmt::Print { signed: false, value: Expr::Lit(1) }])], otherwise: Some(vec![Stmt::Print { signed: false, value: Expr::Lit(0) }]) });
                }
                _ => {
                    let s = self.name("t");
                    // a deci into text says which form (owner, 2026-10-02)
                    let form = if self.cfg.deci_useful && self.rng.chance(50) { "useful.str.utf8" } else { "stored.str.utf8" };
                    body.push(Stmt::AutoLet { name: s.clone(), value: Expr::Conv(form.into(), Box::new(d), None) });
                    body.push(Stmt::Print { signed: false, value: Expr::StrBytes(s) });
                }
            }
        }
        // text and characters (design/conversion.md): into ASCII, bytes
        // read as UTF-8, codes and one-character text, seen as lengths and codes
        if self.rng.chance(60) {
            self.text_part(&mut body);
        }
        // text into an integer
        if self.rng.chance(50) {
            let t = *self.rng.pick(&INTS);
            let k = self.name("k");
            let fb = if t.signed() { -2 } else { 7 };
            body.push(Stmt::AutoLet { name: k.clone(), value: Expr::Conv(t.name().into(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Lit(fb)))) });
            body.push(Stmt::Print { signed: t.signed(), value: Expr::To(if t.signed() { Ty::I64 } else { Ty::U64 }, Box::new(Expr::Var(k))) });
        }
        Stmt::Block { tier: Tier::Default, body }
    }

    /// A signed value divided by -1, printed: the answer is exact and
    /// checked where it lands (owner: a, 15c5c71), so `-128 div.trunc -1`
    /// on an int8 is 128 in `put_i`'s int64. An int64's minimum gives 2^63,
    /// which a uint64 holds (design claude, 2026-10-02): into `put_u`, or into
    /// `put_i`'s int64 as a risk taken, where it does not fit.
    fn min_by_minus_one(&mut self) -> Stmt {
        use BinOp::*;
        let t = *self.rng.pick(&[Ty::I8, Ty::I16, Ty::I32, Ty::I64]);
        let vs = self.vars_of(t);
        let a = if !vs.is_empty() && self.rng.chance(50) && t != Ty::I64 {
            Expr::Var(self.rng.pick(&vs).name.clone())
        } else {
            Expr::To(t, Box::new(Expr::Lit(t.range().lo)))
        };
        let ops: &[BinOp] = if self.cfg.roundings { &[Div, Mod, DivTrunc, ModTrunc, DivCeil, ModCeil, DivNear, ModNear] } else { &[Div, Mod] };
        let op = *self.rng.pick(ops);
        let signed = t != Ty::I64 || self.take_risk();
        Stmt::Print { signed, value: Expr::Bin(op, Box::new(a), Box::new(Expr::Lit(-1))) }
    }

    /// An operator on decis or brain16: `div.near` is ordinary division;
    /// `.floor` / `.ceil` / `.trunc` give a whole quotient, and their `mod`s
    /// what it leaves (`mod.near` is refused there; owner, 5a12c07).
    fn fraction_op(&mut self, with_mod: bool) -> BinOp {
        use BinOp::*;
        let more = if self.cfg.roundings { 1 } else { 0 };
        let m = if with_mod { 3 } else { 0 };
        let mm = if with_mod { more } else { 0 };
        let ops = [(Add, 4), (Sub, 4), (Mul, 4), (DivNear, 4), (Mod, m), (Div, more), (DivCeil, more), (DivTrunc, more), (ModCeil, mm), (ModTrunc, mm)];
        let w: Vec<u64> = ops.iter().map(|o| o.1).collect();
        ops[self.rng.weighted(&w)].0
    }

    /// A `brain16` episode (design/types.md): values from literals, integers,
    /// text, a deci and a bin, worked on with `+ - x div mod xx` (each step
    /// rounded), and seen as integers: `to.int64[…] otherwise -1`, a deci in
    /// either form scaled, a comparison, the length of its text. A risk
    /// taken: a value past `bin16`'s largest converted into one.
    fn brain_episode(&mut self) -> Stmt {
        self.unevaluable = true;
        const LITS: &[&str] = &["0.1", "0.2", "0.5", "1.5", "2.5", "-1.75", "3.0", "12.75", "300.7", "0.001", "1000.0", "-0.3"];
        let names = ["brain16", "bfloat16", "bf16"];
        let mut body = vec![];
        let mut bs: Vec<String> = vec![];
        let first = self.name("w");
        let lit = self.rng.pick(LITS).to_string();
        body.push(Stmt::TypedLet { name: first.clone(), mutable: false, ty: self.rng.pick(&names).to_string(), value: Expr::Num(lit) });
        bs.push(first);
        for _ in 0..1 + self.rng.below(4) {
            let a = Expr::Var(self.rng.pick(&bs).clone());
            let value = match self.rng.below(8) {
                0..=3 => {
                    let op = self.fraction_op(self.cfg.brain_mod);
                    let b = if self.rng.chance(50) { Expr::Var(self.rng.pick(&bs).clone()) } else { Expr::Num(self.rng.pick(LITS).to_string()) };
                    // a divisor of 0 stops both engines alike; keep it rare
                    let b = match (&b, op.division()) {
                        (Expr::Var(_), Some(_)) if !self.take_risk() => Expr::Num("2.5".into()),
                        _ => b,
                    };
                    Expr::Bin(op, Box::new(a), Box::new(b))
                }
                4 => Expr::Bin(BinOp::Pow, Box::new(a), Box::new(Expr::Lit(self.rng.range(0, 3)))),
                5 => {
                    let ints: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
                    match ints.is_empty() {
                        true => Expr::Conv("brain16".into(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Num("1.0".into())))),
                        false => Expr::Conv("brain16".into(), Box::new(Expr::Var(self.rng.pick(&ints).name.clone())), None),
                    }
                }
                6 => Expr::Conv("brain16".into(), Box::new(Expr::Text(self.number_text())), Some(Box::new(Expr::Num(self.rng.pick(LITS).to_string())))),
                _ => {
                    // from a deci or a bin, rounding
                    let (d, ty) = if self.rng.chance(50) { (self.name("d"), "deci64") } else { (self.name("g"), "bin64") };
                    body.push(Stmt::TypedLet { name: d.clone(), mutable: false, ty: ty.into(), value: Expr::Num(self.rng.pick(LITS).to_string()) });
                    Expr::Conv("brain16".into(), Box::new(Expr::Var(d)), None)
                }
            };
            let name = self.name("w");
            let typed = !matches!(value, Expr::Conv(..));
            body.push(if typed { Stmt::TypedLet { name: name.clone(), mutable: false, ty: self.rng.pick(&names).to_string(), value } } else { Stmt::AutoLet { name: name.clone(), value } });
            bs.push(name);
        }
        if self.rng.chance(25) {
            let w = Expr::Var(self.rng.pick(&bs).clone());
            // bin16's largest is 65504: past it, a check site
            let w = if self.take_risk() { Expr::Bin(BinOp::Mul, Box::new(w), Box::new(Expr::Num("70000.0".into()))) } else { w };
            let (v, h) = (self.name("w"), self.name("h"));
            body.push(Stmt::TypedLet { name: v.clone(), mutable: false, ty: "brain16".into(), value: w });
            body.push(Stmt::AutoLet { name: h.clone(), value: Expr::Conv("bin16".into(), Box::new(Expr::Var(v)), None) });
            let d = self.name("d");
            body.push(Stmt::AutoLet { name: d.clone(), value: Expr::Conv("useful.deci64".into(), Box::new(Expr::Var(h)), None) });
            let z = self.name("z");
            body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("int64".into(), Box::new(Expr::Var(d)), Some(Box::new(Expr::Lit(-1)))) });
            body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
        }
        for _ in 0..1 + self.rng.below(3) {
            let w = Expr::Var(self.rng.pick(&bs).clone());
            match self.rng.below(4) {
                0 => {
                    let z = self.name("z");
                    body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("int64".into(), Box::new(w), Some(Box::new(Expr::Lit(-1)))) });
                    body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
                }
                1 => {
                    let form = if self.rng.chance(50) { "useful.deci64" } else { "stored.deci64" };
                    let scale = *self.rng.pick(&["1", "100", "1000000"]);
                    let (d, z) = (self.name("d"), self.name("z"));
                    body.push(Stmt::AutoLet { name: d.clone(), value: Expr::Conv(form.into(), Box::new(w), None) });
                    body.push(Stmt::AutoLet { name: z.clone(), value: Expr::Conv("int64".into(), Box::new(Expr::Bin(BinOp::Mul, Box::new(Expr::Var(d)), Box::new(Expr::Num(scale.into())))), Some(Box::new(Expr::Lit(-1)))) });
                    body.push(Stmt::Print { signed: true, value: Expr::Var(z) });
                }
                2 => {
                    let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Lt, CmpOp::Ge]);
                    let c = Expr::Cmp(op, Box::new(w), Box::new(Expr::Num(self.rng.pick(LITS).to_string())));
                    body.push(Stmt::If { arms: vec![(c, vec![Stmt::Print { signed: false, value: Expr::Lit(1) }])], otherwise: Some(vec![Stmt::Print { signed: false, value: Expr::Lit(0) }]) });
                }
                _ => {
                    let s = self.name("t");
                    let form = if self.rng.chance(50) { "useful.str.utf8" } else { "stored.str.utf8" };
                    body.push(Stmt::AutoLet { name: s.clone(), value: Expr::Conv(form.into(), Box::new(w), None) });
                    body.push(Stmt::Print { signed: false, value: Expr::StrBytes(s) });
                }
            }
        }
        Stmt::Block { tier: Tier::Default, body }
    }

    fn text_part(&mut self, body: &mut Vec<Stmt>) {
        const TEXTS: &[&str] = &["hello", "h\u{e9}llo", "", "Z", "ZZ", "7", "a b", "\u{65e5}\u{672c}", "caf\u{e9}!", "\u{e9}"];
        let q = |n: &str| format!("'{n}'");
        match self.rng.below(5) {
            0 => {
                let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
                let (s, a) = (self.name("t"), self.name("t"));
                body.push(Stmt::TypedLet { name: s.clone(), mutable: false, ty: "str.utf8".into(), value: Expr::Text(t.into()) });
                body.push(Stmt::AutoLet { name: a.clone(), value: Expr::Conv("str.ascii".into(), Box::new(Expr::Var(s)), Some(Box::new(Expr::Text("?".into())))) });
                body.push(Stmt::Print { signed: false, value: Expr::StrBytes(a) });
            }
            1 => {
                // bytes: UTF-8 or not (a stray continuation byte, 255)
                let n = 1 + self.rng.below(6);
                let mut bytes: Vec<u8> = (0..n).map(|_| self.rng.range(97, 122) as u8).collect();
                match self.rng.below(4) {
                    0 => bytes.extend([195, 169]),
                    1 => bytes.push(255),
                    2 => bytes.push(self.rng.range(128, 191) as u8),
                    _ => {}
                }
                let list = if bytes.len() == 1 { format!("[{},]", bytes[0]) } else { format!("[{}]", bytes.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ")) };
                let (l, t) = (self.name("l"), self.name("t"));
                body.push(Stmt::TypedLet { name: l.clone(), mutable: false, ty: "list[uint8]".into(), value: Expr::Src(list) });
                body.push(Stmt::AutoLet { name: t.clone(), value: Expr::Conv("str.utf8".into(), Box::new(Expr::Var(l)), Some(Box::new(Expr::Text("?".into())))) });
                let count = if self.rng.chance(50) { "bytes" } else { "points" };
                body.push(Stmt::Print { signed: false, value: Expr::Src(format!("str.{count}[lend {}]", q(&t))) });
            }
            2 => {
                // a code into a character, not_a_char past the last point
                let code = *self.rng.pick(&[65i128, 97, 233, 26085, 1114111, 1114112, 55296]);
                let c = self.name("c");
                body.push(Stmt::AutoLet { name: c.clone(), value: Expr::Src(format!("char.point.from_code[{code}] otherwise `?`")) });
                body.push(Stmt::Print { signed: false, value: Expr::To(Ty::U64, Box::new(Expr::Src(format!("char.code[{}]", q(&c))))) });
            }
            3 => {
                // one character of text, or not_one_char
                let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
                let c = self.name("c");
                body.push(Stmt::AutoLet { name: c.clone(), value: Expr::Conv("char.point".into(), Box::new(Expr::Text(t.into())), Some(Box::new(Expr::Src("`?`".into())))) });
                body.push(Stmt::Print { signed: false, value: Expr::To(Ty::U64, Box::new(Expr::Src(format!("char.code[{}]", q(&c))))) });
            }
            _ => {
                // a digit's value is a text parse; other encodings never fail
                let d = *self.rng.pick(&["7", "0", "x", "9"]);
                let k = self.name("k");
                body.push(Stmt::AutoLet { name: k.clone(), value: Expr::Conv("int32".into(), Box::new(Expr::Conv("str.utf8".into(), Box::new(Expr::Src(format!("`{d}`"))), None)), Some(Box::new(Expr::Lit(-1)))) });
                body.push(Stmt::Print { signed: true, value: Expr::To(Ty::I64, Box::new(Expr::Var(k))) });
                let t = TEXTS[self.rng.below(TEXTS.len() as u64) as usize];
                let (s, w) = (self.name("t"), self.name("t"));
                body.push(Stmt::TypedLet { name: s.clone(), mutable: false, ty: "str.utf8".into(), value: Expr::Text(t.into()) });
                let enc = *self.rng.pick(&["str.utf16", "str.utf32"]);
                body.push(Stmt::AutoLet { name: w.clone(), value: Expr::Conv(enc.into(), Box::new(Expr::Var(s)), None) });
                body.push(Stmt::Print { signed: false, value: Expr::Src(format!("str.points[lend {}]", q(&w))) });
            }
        }
    }

    fn tier(&mut self, risky: bool) -> Tier {
        match self.rng.below(20) {
            0 | 1 if !risky && !self.positioned => Tier::Check,
            // A vouch, never on anything left unguarded on purpose.
            2 if !risky => if self.rng.chance(50) { Tier::Nocheck } else { Tier::NoChecks },
            _ => Tier::Default,
        }
    }

    fn let_stmt(&mut self) -> Stmt {
        let t = if self.rng.chance(12) { Ty::Bool } else { self.int_ty() };
        let mutability = *self.rng.pick(&[Mutability::Plain, Mutability::Plain, Mutability::Mut, Mutability::Immut]);
        let assignable = mutability != Mutability::Immut && self.rng.chance(70);
        // A strict file says `mut` or `immut` on every declaration.
        let mutability = if self.strict { if assignable { Mutability::Mut } else { Mutability::Immut } } else { mutability };
        self.risky = false;
        self.positioned = false;
        // `promise` covers every value the name will ever hold: each later
        // assignment is fitted into it too.
        let (tr, range) = self.maybe_range(t);
        let (value, promise) = if t != Ty::Bool && !self.enums.is_empty() && self.rng.chance(20) {
            let target = range.unwrap_or_else(|| self.ghost(t));
            (self.match_value(t, target), target)
        } else if t == Ty::Bool {
            let d = self.expr_depth();
            (self.bool_expr(d), Iv::new(0, 1))
        } else if let Some(r) = range {
            // A range type: every value it holds is within it.
            let (e, iv) = self.value_in(t, r);
            (e, if assignable { r } else { iv })
        } else if assignable {
            let g = self.ghost(t);
            let (e, _) = self.value_in(t, g);
            (e, g)
        } else {
            self.int_expr_typed(t)
        };
        let unseen = self.uses_unseen(&value);
        self.positioned |= unseen;
        let tier = self.tier(self.risky);
        let name = self.name("v");
        self.declare(name.clone(), t, promise, assignable);
        if unseen && tr.range.is_none() {
            self.mark_unseen(&name);
        }
        Stmt::Let { tier, name, mutability, ty: tr, value }
    }

    fn assign_stmt(&mut self) -> Stmt {
        let targets: Vec<VarInfo> = self.vars().filter(|v| v.assignable).cloned().collect();
        let v = self.rng.pick(&targets).clone();
        self.risky = false;
        self.positioned = false;
        let value = if v.ty == Ty::Bool { { let d = self.expr_depth(); self.bool_expr(d) } } else { self.value_in(v.ty, v.iv).0 };
        let value = if v.ty != Ty::Bool && !v.unseen && self.uses_unseen(&value) {
            // A seen name keeps what it holds visible: the guard is written
            // out, since a `check` earlier in a loop sees this value next trip.
            self.fit(value, v.ty.range(), v.iv, v.ty).0
        } else {
            value
        };
        self.positioned |= self.uses_unseen(&value);
        let tier = self.tier(self.risky);
        Stmt::Assign { tier, name: v.name, value }
    }

    fn if_stmt(&mut self, depth: u32) -> Stmt {
        let narms = 1 + self.rng.below(3) as usize;
        let has_else = self.rng.chance(50);
        let mut arms = vec![];
        for _ in 0..narms {
            let c = { let d = self.expr_depth(); self.bool_expr(d) };
            let n = self.stmt_count() / 2 + 1;
            let mut body = self.block(n, depth - 1);
            if !has_else {
                if let Some(t) = self.terminal() {
                    body.push(t);
                }
            }
            arms.push((c, body));
        }
        let otherwise = has_else.then(|| {
            let n = self.stmt_count() / 2 + 1;
            self.block(n, depth - 1)
        });
        Stmt::If { arms, otherwise }
    }

    /// A statement that leaves: only ever last in a block, and only in an
    /// `if` with no `else`, so nothing after it is unreachable.
    fn terminal(&mut self) -> Option<Stmt> {
        if !self.rng.chance(20) {
            return None;
        }
        if self.in_loop != LoopKind::None && self.rng.chance(70) {
            return Some(if self.rng.chance(50) { Stmt::Break } else { Stmt::Continue });
        }
        if let Some((t, iv)) = self.in_fn {
            let e = if t == Ty::Bool { self.bool_expr(1) } else { self.value_in(t, iv).0 };
            return Some(Stmt::Return(e));
        }
        None
    }

    fn trips_allowed(&self) -> u64 {
        let left = self.budget.saturating_sub(self.spent) / self.mult.max(1);
        left.min(12)
    }

    fn for_stmt(&mut self, depth: u32) -> Stmt {
        let t = self.int_ty();
        let r = t.range();
        let step: i128 = *self.rng.pick(&[1, 1, 1, 2, 3, 7, -1, -1, -2, -3]);
        let a = step.abs();
        let n = self.rng.below(self.trips_allowed() + 1) as i128;
        let edge = self.rng.chance(35);
        // Choose the first and last value the counter takes, then the stop.
        let (start, stop, iv) = if n == 0 {
            let s = if edge { if step > 0 { r.hi } else { r.lo } } else { self.rng.range(r.lo.max(-50), r.hi.min(50)) };
            let beyond = if step > 0 { s - 1 - self.rng.range(0, 3) } else { s + 1 + self.rng.range(0, 3) };
            if !r.has(beyond) {
                // cannot go past the edge; take a one-trip loop instead
                (s, s, Iv::point(s))
            } else {
                (s, beyond, Iv::point(s))
            }
        } else {
            let span = a * (n - 1);
            let mut over = self.rng.range(0, a - 1);
            let last = if edge {
                if step > 0 { r.hi - self.rng.range(0, a - 1).min(r.hi - r.lo - span) } else { r.lo + self.rng.range(0, a - 1).min(r.hi - r.lo - span) }
            } else if step > 0 {
                self.rng.range((r.lo + span).max(-60), r.hi.min(60).max(r.lo + span))
            } else {
                self.rng.range(r.lo.max(-60).min(r.hi - span), (r.hi - span).min(60))
            };
            let first = if step > 0 { last - span } else { last + span };
            if !self.cfg.range_edge && !r.has(if step > 0 { last + a } else { last - a }) {
                over = 0; // the stop lands on the last value, so the next step is never needed
            }
            let stop = if step > 0 { (last + over).min(r.hi) } else { (last - over).max(r.lo) };
            (first, stop, Iv::new(first.min(last), first.max(last)))
        };
        let trips = if step > 0 { if stop >= start { (stop - start) / a + 1 } else { 0 } } else if start >= stop { (start - stop) / a + 1 } else { 0 };
        let name = self.name("i");
        let perm = self.rng.chance(15);
        let saved = (self.mult, self.in_loop);
        self.mult = self.mult.saturating_mul(trips.max(1) as u64);
        self.in_loop = LoopKind::Loop;
        self.scopes.push(vec![]);
        self.declare(name.clone(), t, iv, false);
        let nb = self.stmt_count() / 2 + 1;
        let body = self.stmts(nb, depth - 1);
        self.scopes.pop();
        (self.mult, self.in_loop) = saved;
        if perm {
            self.declare(name.clone(), t, iv.join(Iv::point(start)), false);
        }
        let step = if step == 1 && self.rng.chance(60) { None } else { Some(step) };
        let cheat = if self.cfg.no_cheat && self.rng.chance(15) { 1 + self.rng.below(2) as u8 } else { 0 };
        Stmt::For { perm, cheat, name, ty: t, start: Expr::Lit(start), stop: Expr::Lit(stop), step, body }
    }

    /// `while` or `loop`, driven by a counter the body cannot touch.
    /// Returns a block holding the counter's `let` and the loop.
    fn counted_loop(&mut self, depth: u32, is_loop: bool) -> Stmt {
        let t = self.int_ty();
        let k = self.rng.below(self.trips_allowed() + 1).min(t.max() as u64) as i128;
        let name = self.name("w");
        let counter = Stmt::Let { tier: Tier::Default, name: name.clone(), mutability: Mutability::Mut, ty: t.into(), value: Expr::Lit(k) };
        let dec = Stmt::Assign { tier: Tier::Default, name: name.clone(), value: Expr::Bin(BinOp::Sub, Box::new(Expr::Var(name.clone())), Box::new(Expr::Lit(1))) };
        self.scopes.push(vec![]);
        self.declare(name.clone(), t, Iv::new(0, k), false);
        let saved = (self.mult, self.in_loop);
        self.mult = self.mult.saturating_mul(k.max(1) as u64);
        self.in_loop = LoopKind::Loop;
        let nb = self.stmt_count() / 2 + 1;
        let rest = self.block(nb, depth - 1);
        (self.mult, self.in_loop) = saved;
        let var = || Box::new(Expr::Var(name.clone()));
        let the_loop = if is_loop {
            let stop = Expr::Cmp(if self.rng.chance(50) { CmpOp::Eq } else { CmpOp::Le }, var(), Box::new(Expr::Lit(0)));
            let mut body = vec![Stmt::If { arms: vec![(stop, vec![Stmt::Break])], otherwise: None }, dec];
            body.extend(rest);
            Stmt::Loop { body }
        } else {
            let mut cond = Expr::Cmp(CmpOp::Gt, var(), Box::new(Expr::Lit(0)));
            if self.rng.chance(25) {
                let extra = self.bool_expr(1);
                cond = Expr::And(Box::new(cond), Box::new(extra));
            }
            let mut body = vec![dec];
            body.extend(rest);
            Stmt::While { cond, body }
        };
        self.scopes.pop();
        Stmt::Block { tier: Tier::Default, body: vec![counter, the_loop] }
    }

    // ------------------------------------------------------------ expressions

    /// An expression of type `t` whose value lies in `target` (⊆ t's range).
    fn value_in(&mut self, t: Ty, target: Iv) -> (Expr, Iv) {
        if t == Ty::Bool {
            return ({ let d = self.expr_depth(); self.bool_expr(d) }, Iv::new(0, 1));
        }
        let d = self.expr_depth();
        let (e, iv) = self.int_expr(t, d);
        self.fit(e, iv, target, t)
    }

    /// Like `int_expr`, but never literal-only (so its type is its own, not
    /// its context's).
    fn int_expr_typed(&mut self, t: Ty) -> (Expr, Iv) {
        let d = self.expr_depth();
        let (e, iv) = self.int_expr(t, d);
        if is_literal_only(&e) {
            if let Some((v, viv)) = self.typed_leaf(t) {
                return self.combine_add(t, e, iv, v, viv);
            }
        }
        (e, iv)
    }

    /// `a + b` with `b` fitted so the sum stays in `t`; `a` is literal-only.
    fn combine_add(&mut self, t: Ty, a: Expr, ia: Iv, b: Expr, ib: Iv) -> (Expr, Iv) {
        let r = t.range();
        let (lo, hi) = (r.lo - ia.lo, r.hi - ia.hi);
        let Some(allowed) = (lo <= hi).then(|| Iv::new(lo, hi)).and_then(|x| x.meet(r)) else { return (b, ib) };
        let (b, ib) = self.fit(b, ib, allowed, t);
        (Expr::Bin(BinOp::Add, Box::new(a), Box::new(b)), Iv::new(ia.lo + ib.lo, ia.hi + ib.hi))
    }

    fn typed_leaf(&mut self, t: Ty) -> Option<(Expr, Iv)> {
        let same = self.vars_of(t);
        if !same.is_empty() {
            let v = self.rng.pick(&same).clone();
            return Some((Expr::Var(v.name), v.iv));
        }
        let others: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
        if others.is_empty() {
            return None;
        }
        let v = self.rng.pick(&others).clone();
        Some(self.convert(Expr::Var(v.name), v.iv, v.ty, t))
    }

    /// `to.<to>[e]`, with `e` first folded into what `to` holds (unless risky).
    fn convert(&mut self, e: Expr, iv: Iv, from: Ty, to: Ty) -> (Expr, Iv) {
        let both = from.range().meet(to.range()).unwrap();
        let (e, iv) = if iv.within(to.range()) || self.take_risk() {
            (e, iv.meet(to.range()).unwrap_or(to.range()))
        } else {
            self.fit(e, iv, both, from)
        };
        (Expr::To(to, Box::new(e)), iv)
    }

    fn take_risk(&mut self) -> bool {
        if self.in_check > 0 {
            return false;
        }
        let r = self.rng.below(1000) < self.cfg.risk;
        if r {
            self.risky = true;
        }
        r
    }

    /// Fold `e` (of type `t`, value in `iv`) into `target`. The result's
    /// interval is within `target`, possibly narrower.
    fn fit(&mut self, e: Expr, iv: Iv, target: Iv, t: Ty) -> (Expr, Iv) {
        if iv.within(target) {
            return (e, iv);
        }
        let mut tg = target;
        if tg.lo == t.min() && t.signed() {
            tg.lo += 1; // `- 128` would need 128 as an int8 literal
            if tg.lo > tg.hi {
                return (Expr::Lit(t.min()), Iv::point(t.min()));
            }
        }
        if tg.width() > t.max() {
            let w = t.max();
            let lo = self.rng.range(tg.lo, tg.hi - w + 1);
            tg = Iv::new(lo, lo + w - 1);
        }
        if let Expr::Lit(_) = e {
            let v = self.rng.range(tg.lo, tg.hi);
            return (Expr::Lit(v), Iv::point(v));
        }
        let w = tg.width();
        let m = Expr::Bin(BinOp::Mod, Box::new(e), Box::new(Expr::Lit(w)));
        let e = if tg.lo == 0 {
            m
        } else if tg.lo > 0 {
            Expr::Bin(BinOp::Add, Box::new(m), Box::new(Expr::Lit(tg.lo)))
        } else {
            Expr::Bin(BinOp::Sub, Box::new(m), Box::new(Expr::Lit(-tg.lo)))
        };
        (e, tg)
    }

    fn literal(&mut self, t: Ty) -> (Expr, Iv) {
        let r = t.range();
        let pick = if self.cfg.edge && self.rng.chance(60) { 0 } else { self.rng.below(3) };
        let v = match pick {
            0 => {
                let edges = [r.lo, r.lo + 1, -1, 0, 1, 2, r.hi - 1, r.hi, r.hi / 2, r.hi / 2 + 1, r.lo / 2];
                let ok: Vec<i128> = edges.into_iter().filter(|v| r.has(*v)).collect();
                *self.rng.pick(&ok)
            }
            1 => self.rng.range(r.lo.max(-20), r.hi.min(20)),
            _ => self.rng.range(r.lo, r.hi),
        };
        (Expr::Lit(v), Iv::point(v))
    }

    fn call(&mut self, t: Ty) -> Option<(Expr, Iv)> {
        let fits: Vec<Sig> = self.sigs[..self.callable]
            .iter()
            .filter(|s| s.ret == t && (self.in_check == 0 || self.strict || s.ret_ranged) && self.spent + s.cost.saturating_mul(self.mult) <= self.budget)
            .cloned()
            .collect();
        if fits.is_empty() {
            return None;
        }
        let s = self.rng.pick(&fits).clone();
        self.spent += s.cost.saturating_mul(self.mult);
        let mut args = vec![];
        for (_, pt, piv) in &s.params {
            let (a, _) = if *pt == Ty::Bool { (self.bool_expr(1), Iv::new(0, 1)) } else { self.value_in(*pt, *piv) };
            args.push(a);
        }
        Some((Expr::Call(s.name.clone(), args), s.ret_iv))
    }

    fn leaf(&mut self, t: Ty) -> (Expr, Iv) {
        let same = self.vars_of(t);
        let others = self.vars().any(|v| v.ty != t && v.ty != Ty::Bool);
        let can_call = self.sigs[..self.callable].iter().any(|s| s.ret == t && (self.in_check == 0 || self.strict || s.ret_ranged));
        let readable = self.readable(Some(t));
        let items: Vec<VarInfo> = self.vars().filter(|v| v.ty == t && self.through_to(v)).cloned().collect();
        // A lane's range is the generator's own: never inside `check`.
        let simds = if self.in_check > 0 { vec![] } else { self.simd_vars(Some(t)) };
        let has_list = !self.lists().is_empty();
        let w = [
            if same.is_empty() { 0 } else { 6 },
            if others { 2 } else { 0 },
            3,
            if can_call { 1 } else { 0 },
            if readable.is_empty() { 0 } else { 3 },
            if t == Ty::U64 && has_list && self.in_check == 0 { 1 } else { 0 },
            if items.is_empty() { 0 } else { 4 },
            if simds.is_empty() { 0 } else { 3 },
        ];
        match self.rng.weighted(&w) {
            4 => {
                let v = self.rng.pick(&readable).clone();
                let p = self.position(&v);
                (Expr::At(v.name, Box::new(p)), v.iv)
            }
            7 => {
                let v = self.rng.pick(&simds).clone();
                self.simd_read(v, t)
            }
            6 => {
                let v = self.rng.pick(&items).clone();
                (Expr::To(t, Box::new(Expr::Var(v.name))), v.iv)
            }
            5 => {
                let ls = self.lists();
                let v = self.rng.pick(&ls).clone();
                let l = v.list.unwrap();
                (Expr::Len(v.name), Iv::new(l.lo as i128, l.cap as i128))
            }
            0 => {
                let v = self.rng.pick(&same).clone();
                (Expr::Var(v.name), v.iv)
            }
            1 => {
                let os: Vec<VarInfo> = self.vars().filter(|v| v.ty != t && v.ty != Ty::Bool).cloned().collect();
                let v = self.rng.pick(&os).clone();
                self.convert(Expr::Var(v.name), v.iv, v.ty, t)
            }
            3 => self.call(t).unwrap_or_else(|| self.literal(t)),
            _ => self.literal(t),
        }
    }

    fn int_expr(&mut self, t: Ty, depth: u32) -> (Expr, Iv) {
        if depth == 0 || self.rng.chance(30) {
            return self.leaf(t);
        }
        let s = t.signed();
        let has_var = !self.vars_of(t).is_empty();
        use BinOp::*;
        let more = if self.cfg.roundings { 1 } else { 0 };
        let ops: [(Option<BinOp>, u64); 22] = [
            (Some(Add), 6),
            (Some(Sub), 6),
            (Some(Mul), 5),
            (Some(Div), 3),
            (Some(Mod), 3),
            (Some(Pow), if self.cfg.pow { 1 } else { 0 }),
            (Some(AndBit), 1),
            (Some(OrBit), 1),
            (Some(Xor), 1),
            (Some(Shl), 1),
            (Some(Shr), 1),
            (Some(WAdd), if has_var { 1 } else { 0 }),
            (Some(WSub), if has_var { 1 } else { 0 }),
            (Some(WMul), if has_var { 1 } else { 0 }),
            (None, if s { 2 } else { 0 }),                  // negation
            (Some(Mod), 0),
            (Some(DivTrunc), more),
            (Some(ModTrunc), more),
            (Some(DivCeil), more),
            (Some(ModCeil), more),
            (Some(DivNear), more),
            (Some(ModNear), more),
        ];
        let w: Vec<u64> = ops.iter().map(|o| o.1).collect();
        let pick = ops[self.rng.weighted(&w)].0;
        let Some(op) = pick else { return self.neg(t, depth) };
        if matches!(op, AndBit | OrBit | Xor) && has_var && self.rng.chance(15) {
            // not.bit: of a name, or of any typed operand once the width may
            // come from context
            let (a, ia) = if self.cfg.width_from_context && self.rng.chance(50) {
                let (e, iv) = self.int_expr_typed(t);
                (if self.cfg.sized_bits && !matches!(e, Expr::Var(_) | Expr::To(..)) { Expr::To(t, Box::new(e)) } else { e }, iv)
            } else {
                let vs = self.vars_of(t);
                let v = self.rng.pick(&vs).clone();
                (Expr::Var(v.name), v.iv)
            };
            let iv = if s { Iv::new(-ia.hi - 1, -ia.lo - 1) } else { Iv::new(t.max() - ia.hi, t.max() - ia.lo) };
            return (Expr::NotBit(Box::new(a)), iv);
        }
        let (a, ia) = if matches!(op, WAdd | WSub | WMul) && !(self.cfg.width_from_context && self.rng.chance(50)) {
            let vs = self.vars_of(t);
            let v = self.rng.pick(&vs).clone();
            (Expr::Var(v.name), v.iv)
        } else if matches!(op, WAdd | WSub | WMul) {
            self.int_expr_typed(t)
        } else {
            self.int_expr(t, depth - 1)
        };
        let (b, ib) = self.int_expr(t, depth - 1);
        // `shl`'s left operand is a name or a conversion: CCA takes the width
        // only from those (question open with the design session, 2026-09-23).
        let named = matches!(a, Expr::Var(_) | Expr::To(..));
        let bare_left = op == Shl && !(self.cfg.width_from_context && !is_literal_only(&a));
        let (op, a, ia) = if matches!(op, Shl | Shr) && !named && bare_left {
            match self.typed_leaf(t) {
                Some((v, iv)) => (op, v, iv),
                None => (Shr, a, ia),
            }
        } else {
            (op, a, ia)
        };
        self.binary(t, op, a, ia, b, ib)
    }

    fn neg(&mut self, t: Ty, depth: u32) -> (Expr, Iv) {
        let (a, ia) = self.int_expr(t, depth - 1);
        let r = t.range();
        let (a, ia) = if ia.lo > r.lo || self.take_risk() { (a, ia) } else { self.fit(a, ia, Iv::new(r.lo + 1, r.hi), t) };
        let iv = Iv::new(-ia.hi, -ia.lo).meet(r).unwrap_or(r);
        (Expr::Neg(Box::new(a)), iv)
    }

    fn binary(&mut self, t: Ty, op: BinOp, a: Expr, ia: Iv, b: Expr, ib: Iv) -> (Expr, Iv) {
        use BinOp::*;
        let bitty = matches!(op, AndBit | OrBit | Xor | Shl | Shr | WAdd | WSub | WMul);
        let size = |e: Expr| -> Expr {
            match e {
                Expr::Var(_) | Expr::To(..) | Expr::Lit(_) => e,
                e => Expr::To(t, Box::new(e)),
            }
        };
        let (a, b) = if bitty && self.cfg.sized_bits { (size(a), size(b)) } else { (a, b) };
        // Bare numbers on both sides leave the operator no width at all: no
        // ruling yet on what that means (design claude, 2026-09-24), so one
        // side becomes a typed name.
        let (a, ia) = if bitty && is_literal_only(&a) && is_literal_only(&b) {
            match self.typed_leaf(t) {
                Some((v, iv)) => (v, iv),
                // Nothing typed in scope: no operator at all (seed 444).
                None => return (a, ia),
            }
        } else {
            (a, ia)
        };
        let r = t.range();
        let bits = t.bits() as i128;
        let risk = self.take_risk();
        let mk = |a: Expr, b: Expr| Expr::Bin(op, Box::new(a), Box::new(b));
        let clip = |iv: Option<Iv>| iv.and_then(|x| x.meet(r)).unwrap_or(r);
        match op {
            Add | Sub => {
                let f = |x: i128, y: i128| Some(if op == Add { x + y } else { x - y });
                let out = corners(ia, ib, &f).unwrap();
                if out.within(r) || risk {
                    return (mk(a, b), clip(Some(out)));
                }
                // allowed b given a
                let (lo, hi) = if op == Add { (r.lo - ia.lo, r.hi - ia.hi) } else { (ia.hi - r.hi, ia.lo - r.lo) };
                let (a, ia) = if lo > hi || Iv::new(lo.min(hi), hi.max(lo)).meet(r).is_none() {
                    let half = Iv::new(r.lo / 2, r.hi / 2);
                    self.fit(a, ia, half, t)
                } else {
                    (a, ia)
                };
                let (lo, hi) = if op == Add { (r.lo - ia.lo, r.hi - ia.hi) } else { (ia.hi - r.hi, ia.lo - r.lo) };
                let allowed = if lo <= hi { Iv::new(lo, hi).meet(r) } else { None };
                let Some(allowed) = allowed else {
                    // fall back to something that certainly fits
                    return (a, ia);
                };
                let (b, ib) = self.fit(b, ib, allowed, t);
                let out = corners(ia, ib, &f).unwrap();
                (mk(a, b), clip(Some(out)))
            }
            Mul => {
                let f = |x: i128, y: i128| x.checked_mul(y);
                let out = corners(ia, ib, &f);
                if out.map_or(false, |o| o.within(r)) || risk {
                    return (mk(a, b), clip(out));
                }
                let s = isqrt(r.hi);
                let bx = if t.signed() { Iv::new(-s, s) } else { Iv::new(0, s) };
                let (a, ia) = self.fit(a, ia, bx, t);
                let (b, ib) = self.fit(b, ib, bx, t);
                (mk(a, b), clip(corners(ia, ib, &f)))
            }
            Div | Mod | DivTrunc | ModTrunc | DivCeil | ModCeil | DivNear | ModNear => {
                let bad_zero = ib.has(0);
                let bad_min = t.signed() && ia.has(r.lo) && ib.has(-1);
                let (b, ib) = if (bad_zero || bad_min) && !risk {
                    let k = if self.rng.chance(50) { self.rng.range(1, 10) } else { self.rng.range(1, r.hi) };
                    let neg = t.signed() && !ia.has(r.lo) && self.rng.chance(25);
                    let tg = if neg { Iv::new(-k, -1) } else { Iv::new(1, k) };
                    self.fit(b, ib, tg, t)
                } else {
                    (b, ib)
                };
                if ib.has(0) {
                    // risky: division by zero possible
                    return (mk(a, b), r);
                }
                let out = if op.division() == Some(false) {
                    corners(ia, ib, |x, y| op.quotient(x, y))
                } else {
                    let (lo, hi) = op.mod_bounds((ia.lo, ia.hi), (ib.lo, ib.hi));
                    Some(Iv::new(lo, hi))
                };
                (mk(a, b), clip(out))
            }
            Pow => {
                let (a, ia) = if risk { (a, ia) } else { self.fit(a, ia, if t.signed() { Iv::new(-3, 3) } else { Iv::new(0, 3) }, t) };
                let mut emax = 0i128;
                while 3i128.pow(emax as u32 + 1) <= r.hi {
                    emax += 1;
                }
                let (b, ib) = if ib.lo >= 0 && ib.hi <= emax { (b, ib) } else { self.fit(b, ib, Iv::new(0, emax), t) };
                if ia.width() > 7 || ib.lo < 0 || ib.hi > 64 {
                    return (mk(a, b), r);
                }
                let mut lo = i128::MAX;
                let mut hi = i128::MIN;
                for x in ia.lo..=ia.hi {
                    for e in ib.lo..=ib.hi {
                        let v = x.checked_pow(e as u32).unwrap_or(i128::MAX);
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
                (mk(a, b), clip(Some(Iv::new(lo, hi))))
            }
            AndBit | OrBit | Xor => {
                if ia.lo >= 0 && ib.lo >= 0 {
                    let top = ia.hi.max(ib.hi);
                    let mask = if top == 0 { 0 } else { (1i128 << (128 - top.leading_zeros())) - 1 };
                    let iv = if op == AndBit { Iv::new(0, ia.hi.min(ib.hi)) } else { Iv::new(0, mask) };
                    (mk(a, b), clip(Some(iv)))
                } else {
                    (mk(a, b), r)
                }
            }
            Shl | Shr => {
                let (b, ib) = if (ib.lo >= 0 && ib.hi < bits) || risk { (b, ib) } else {
                    let k = self.rng.range(0, bits - 1);
                    if self.rng.chance(60) { (Expr::Lit(k), Iv::point(k)) } else { self.fit(b, ib, Iv::new(0, bits - 1), t) }
                };
                if ib.lo < 0 || ib.hi >= bits {
                    return (mk(a, b), r);
                }
                if op == Shl {
                    // Bound the shift, not the operand, so the operand stays a name.
                    let fits = |k: i128| corners(ia, Iv::new(0, k), |x, s| x.checked_mul(1i128 << s)).map_or(false, |o| o.within(r));
                    // A risk taken: often a small, valid count on a value that
                    // may be too big to shift, so only the answer can leave the
                    // type (a wrong layer-3 proof hid there, seed 9148).
                    let (b, ib) = if risk && ib.hi >= 1 && self.rng.chance(50) {
                        let k = self.rng.range(1, 3.min(bits - 1));
                        (Expr::Lit(k), Iv::point(k))
                    } else {
                        (b, ib)
                    };
                    let (b, ib) = if fits(ib.hi) || risk { (b, ib) } else {
                        let mut k = 0;
                        while k + 1 < bits && fits(k + 1) {
                            k += 1;
                        }
                        self.fit(b, ib, Iv::new(0, k), t)
                    };
                    let out = corners(ia, ib, |x, k| x.checked_mul(1i128 << k));
                    (mk(a, b), clip(out))
                } else if ia.lo >= 0 {
                    (mk(a, b), Iv::new(ia.lo >> ib.hi, ia.hi >> ib.lo))
                } else {
                    (mk(a, b), r)
                }
            }
            WAdd | WSub | WMul => (mk(a, b), r),
        }
    }

    fn bool_expr(&mut self, depth: u32) -> Expr {
        let bools = self.vars_of(Ty::Bool);
        let svs = if self.in_check > 0 { vec![] } else { self.simd_vars(None) };
        if !svs.is_empty() && self.rng.chance(8) {
            let v = self.rng.pick(&svs).clone();
            let other = self.simd_operand(v.ty, v.iv);
            let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Ne, CmpOp::Lt, CmpOp::Gt, CmpOp::Le, CmpOp::Ge]);
            let f = if self.rng.chance(50) { "all" } else { "any" };
            return Expr::SimdFold(f, Box::new(Expr::Cmp(op, Box::new(Expr::Var(v.name)), Box::new(other.0))));
        }
        let evs = self.enum_vars();
        if !evs.is_empty() && self.rng.chance(8) {
            let v = self.rng.pick(&evs).clone();
            let c = self.case_value(v.en.unwrap(), false);
            let op = if self.rng.chance(50) { CmpOp::Eq } else { CmpOp::Ne };
            return Expr::Cmp(op, Box::new(Expr::Var(v.name)), Box::new(c));
        }
        if self.rng.chance(6) {
            if let Some(g) = self.guarded() {
                return g;
            }
        }
        if depth == 0 || self.rng.chance(35) {
            let w = [1, if bools.is_empty() { 0 } else { 3 }, 6];
            return match self.rng.weighted(&w) {
                0 => Expr::Bool(self.rng.chance(50)),
                1 => Expr::Var(self.rng.pick(&bools).name.clone()),
                _ => self.comparison(),
            };
        }
        match self.rng.below(5) {
            0 | 1 => {
                let (a, b) = (Box::new(self.bool_expr(depth - 1)), Box::new(self.bool_expr(depth - 1)));
                if self.rng.chance(30) { Expr::AndThen(a, b) } else { Expr::And(a, b) }
            }
            2 | 3 => {
                let (a, b) = (Box::new(self.bool_expr(depth - 1)), Box::new(self.bool_expr(depth - 1)));
                if self.rng.chance(30) { Expr::OrElse(a, b) } else { Expr::Or(a, b) }
            }
            _ => Expr::Not(Box::new(self.bool_expr(depth - 1))),
        }
    }

    /// A division that only its guard makes safe: `'v' !== 0 and.then
    /// (k div 'v') < m`, or `'v' == 0 or.else …` (owner, 2026-09-25). The
    /// right side is left unguarded on purpose; Precog must see the left.
    fn guarded(&mut self) -> Option<Expr> {
        let vs: Vec<VarInfo> = self.vars().filter(|v| v.ty != Ty::Bool).cloned().collect();
        if vs.is_empty() {
            return None;
        }
        let v = self.rng.pick(&vs).clone();
        let t = v.ty;
        let r = t.range();
        // The numerator: never the type's minimum, so `div -1` cannot overflow.
        let k = self.rng.range(r.lo.max(-50), r.hi.min(50));
        let op = if self.rng.chance(50) { BinOp::Div } else { BinOp::Mod };
        let var = || Box::new(Expr::Var(v.name.clone()));
        // Sometimes the excluded value sits inside the range, `'v' == j
        // or.else … div ('v' - j)`, where an interval cannot leave it out
        // (compiler claude, 2026-09-30); only below 64 bits, so 'v' - j
        // cannot pass the 64-bit limit in between.
        let j = if t.bits() < 64 && self.rng.chance(35) { self.rng.range(r.lo.max(-50), r.hi.min(50)) } else { 0 };
        if j != 0 {
            let q = Expr::Bin(op, Box::new(Expr::Lit(k)), Box::new(Expr::Bin(BinOp::Sub, var(), Box::new(Expr::Lit(j)))));
            let m = self.rng.range(-3, 3);
            let right = Box::new(Expr::Cmp(CmpOp::Lt, Box::new(q), Box::new(Expr::Lit(m))));
            return Some(if self.rng.chance(50) {
                Expr::AndThen(Box::new(Expr::Cmp(CmpOp::Ne, var(), Box::new(Expr::Lit(j)))), right)
            } else {
                Expr::OrElse(Box::new(Expr::Cmp(CmpOp::Eq, var(), Box::new(Expr::Lit(j)))), right)
            });
        }
        let q = Expr::Bin(op, Box::new(Expr::Lit(k)), var());
        let m = self.rng.range(-3, 3).max(r.lo);
        let cmp = *self.rng.pick(&[CmpOp::Lt, CmpOp::Ge, CmpOp::Eq, CmpOp::Ne]);
        let right = Box::new(Expr::Cmp(cmp, Box::new(q), Box::new(Expr::Lit(m))));
        Some(if self.rng.chance(50) {
            Expr::AndThen(Box::new(Expr::Cmp(CmpOp::Ne, var(), Box::new(Expr::Lit(0)))), right)
        } else {
            Expr::OrElse(Box::new(Expr::Cmp(CmpOp::Eq, var(), Box::new(Expr::Lit(0)))), right)
        })
    }

    fn comparison(&mut self) -> Expr {
        let t = self.int_ty();
        let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Ne, CmpOp::Lt, CmpOp::Gt, CmpOp::Le, CmpOp::Ge]);
        let d = self.expr_depth().min(2);
        let (a, _) = self.int_expr(t, d);
        // A comparison compares true values (owner, dc83c05), so its sides
        // may differ in type; in edge mode they often do.
        let tb = if self.cfg.edge && self.rng.chance(40) { self.int_ty() } else { t };
        let (b, _) = self.int_expr(tb, d);
        if is_literal_only(&a) && is_literal_only(&b) {
            match self.typed_leaf(t) {
                Some((v, _)) => return Expr::Cmp(op, Box::new(v), Box::new(b)),
                None => return Expr::Bool(self.rng.chance(50)),
            }
        }
        Expr::Cmp(op, Box::new(a), Box::new(b))
    }
}

