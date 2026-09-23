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
use crate::printer::printer;
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
}

impl Default for Config {
    fn default() -> Config {
        Config { size: 3, inputs: 0, risk: 5, pow: false, range_edge: false, width_from_context: false, ranges: true }
    }
}

#[derive(Clone, Debug)]
struct VarInfo {
    name: String,
    ty: Ty,
    iv: Iv,
    assignable: bool,
}

#[derive(Clone, Debug)]
struct Sig {
    name: String,
    params: Vec<(String, Ty, Iv)>,
    ret: Ty,
    ret_iv: Iv,
    cost: u64,
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
}

const MAIN_BUDGET: u64 = 20_000;
const FN_BUDGET: u64 = 400;

pub fn generate(seed: u64, cfg: &Config) -> Program {
    let mut root = Rng::new(seed);
    let mut prng = root.split();
    let printer = printer(&mut prng);
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
    };
    let nfns = g.rng.below(cfg.size as u64 + 1) as usize;
    let mut funcs = vec![];
    for i in 0..nfns {
        funcs.push(g.function(i));
    }
    // MAIN
    g.callable = g.sigs.len();
    g.in_fn = None;
    g.budget = MAIN_BUDGET;
    g.spent = 0;
    g.mult = 1;
    g.scopes = vec![vec![]];
    let mut inputs = vec![];
    for k in 0..cfg.inputs {
        let name = format!("in{}", k + 1);
        inputs.push(Input { name: name.clone(), var: format!("CHK_IN{}", k + 1) });
        g.declare(name, Ty::U64, Ty::U64.range(), false);
    }
    let mut main = vec![];
    // A few typed names first, so comparisons always have something typed.
    for _ in 0..1 + g.rng.below(3) {
        main.push(g.let_stmt());
    }
    let n = g.stmt_count();
    main.extend(g.stmts(n, g.depth()));
    Program { seed, printer, inputs, funcs, main, loose }
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
        Expr::Var(_) | Expr::Call(..) | Expr::To(..) => false,
        Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) => is_literal_only(a) && is_literal_only(b),
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
        self.scopes.last_mut().unwrap().push(VarInfo { name, ty, iv, assignable });
    }

    fn vars(&self) -> impl Iterator<Item = &VarInfo> {
        self.scopes.iter().flatten()
    }

    fn vars_of(&self, t: Ty) -> Vec<VarInfo> {
        self.vars().filter(|v| v.ty == t).cloned().collect()
    }

    fn int_ty(&mut self) -> Ty {
        *self.rng.pick(&INTS)
    }

    /// A sub-interval of `t` to promise for a name, or the whole type.
    fn ghost(&mut self, t: Ty) -> Iv {
        let r = t.range();
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
        if lo.is_none() && hi.is_none() || (lo == Some(r.lo) && hi == Some(r.hi)) {
            return (t.into(), None);
        }
        let iv = Iv::new(lo.unwrap_or(r.lo), hi.unwrap_or(r.hi));
        (TyR { base: t, range: Some((lo, hi)) }, Some(iv))
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
        }
        let n = self.stmt_count() / 2 + 1;
        let mut body = self.stmts(n, self.depth().saturating_sub(1).max(1));
        let (e, _) = self.value_in(ret, ret_iv);
        body.push(Stmt::Return(e));
        self.sigs.push(Sig { name: name.clone(), params: params.clone(), ret, ret_iv, cost: self.spent.max(1) });
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
        ];
        match self.rng.weighted(&w) {
            0 => self.let_stmt(),
            1 => self.assign_stmt(),
            2 => self.if_stmt(depth),
            3 => self.for_stmt(depth),
            4 => self.counted_loop(depth, false),
            5 => self.counted_loop(depth, true),
            6 => {
                let t = self.int_ty();
                let (e, _) = self.int_expr_typed(t);
                Stmt::Print { signed: t.signed(), value: e }
            }
            _ => {
                let tier = if self.rng.chance(50) { Tier::Check } else { Tier::Default };
                let n = self.stmt_count() / 2 + 1;
                Stmt::Block { tier, body: self.block(n, depth - 1) }
            }
        }
    }

    fn tier(&mut self, risky: bool) -> Tier {
        match self.rng.below(20) {
            0 | 1 => Tier::Check,
            2 if !risky => Tier::Nocheck,
            _ => Tier::Default,
        }
    }

    fn let_stmt(&mut self) -> Stmt {
        let t = if self.rng.chance(12) { Ty::Bool } else { self.int_ty() };
        let mutability = *self.rng.pick(&[Mutability::Plain, Mutability::Plain, Mutability::Mut, Mutability::Immut]);
        let assignable = mutability != Mutability::Immut && self.rng.chance(70);
        self.risky = false;
        // `promise` covers every value the name will ever hold: each later
        // assignment is fitted into it too.
        let (tr, range) = self.maybe_range(t);
        let (value, promise) = if t == Ty::Bool {
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
        let tier = self.tier(self.risky);
        let name = self.name("v");
        self.declare(name.clone(), t, promise, assignable);
        Stmt::Let { tier, name, mutability, ty: tr, value }
    }

    fn assign_stmt(&mut self) -> Stmt {
        let targets: Vec<VarInfo> = self.vars().filter(|v| v.assignable).cloned().collect();
        let v = self.rng.pick(&targets).clone();
        self.risky = false;
        let value = if v.ty == Ty::Bool { { let d = self.expr_depth(); self.bool_expr(d) } } else { self.value_in(v.ty, v.iv).0 };
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
        Stmt::For { perm, name, ty: t, start: Expr::Lit(start), stop: Expr::Lit(stop), step, body }
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
        let v = match self.rng.below(3) {
            0 => {
                let edges = [r.lo, r.lo + 1, -1, 0, 1, 2, r.hi - 1, r.hi];
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
            .filter(|s| s.ret == t && self.spent + s.cost.saturating_mul(self.mult) <= self.budget)
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
        let can_call = self.sigs[..self.callable].iter().any(|s| s.ret == t);
        let w = [if same.is_empty() { 0 } else { 6 }, if others { 2 } else { 0 }, 3, if can_call { 1 } else { 0 }];
        match self.rng.weighted(&w) {
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
        let ops: [(Option<BinOp>, u64); 16] = [
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
        ];
        let w: Vec<u64> = ops.iter().map(|o| o.1).collect();
        let pick = ops[self.rng.weighted(&w)].0;
        let Some(op) = pick else { return self.neg(t, depth) };
        if matches!(op, AndBit | OrBit | Xor) && has_var && self.rng.chance(15) {
            // not.bit: of a name, or of any typed operand once the width may
            // come from context
            let (a, ia) = if self.cfg.width_from_context && self.rng.chance(50) {
                self.int_expr_typed(t)
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
        let (op, a, ia) = if op == Shl && !named && !(self.cfg.width_from_context && !is_literal_only(&a)) {
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
            Div | Mod => {
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
                let out = if op == Div {
                    corners(ia, ib, |x, y| Some(floor_div(x, y)))
                } else if ib.lo > 0 {
                    Some(if ia.lo >= 0 && ia.hi < ib.lo { ia } else { Iv::new(0, ib.hi - 1) })
                } else {
                    Some(Iv::new(ib.lo + 1, 0))
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
        if depth == 0 || self.rng.chance(35) {
            let w = [1, if bools.is_empty() { 0 } else { 3 }, 6];
            return match self.rng.weighted(&w) {
                0 => Expr::Bool(self.rng.chance(50)),
                1 => Expr::Var(self.rng.pick(&bools).name.clone()),
                _ => self.comparison(),
            };
        }
        match self.rng.below(5) {
            0 | 1 => Expr::And(Box::new(self.bool_expr(depth - 1)), Box::new(self.bool_expr(depth - 1))),
            2 | 3 => Expr::Or(Box::new(self.bool_expr(depth - 1)), Box::new(self.bool_expr(depth - 1))),
            _ => Expr::Not(Box::new(self.bool_expr(depth - 1))),
        }
    }

    fn comparison(&mut self) -> Expr {
        let t = self.int_ty();
        let op = *self.rng.pick(&[CmpOp::Eq, CmpOp::Ne, CmpOp::Lt, CmpOp::Gt, CmpOp::Le, CmpOp::Ge]);
        let d = self.expr_depth().min(2);
        let (a, _) = self.int_expr(t, d);
        let (b, _) = self.int_expr(t, d);
        if is_literal_only(&a) && is_literal_only(&b) {
            match self.typed_leaf(t) {
                Some((v, _)) => return Expr::Cmp(op, Box::new(v), Box::new(b)),
                None => return Expr::Bool(self.rng.chance(50)),
            }
        }
        Expr::Cmp(op, Box::new(a), Box::new(b))
    }
}

