//! Attack (b) of the Precog oracle (design/precog.md): an independent,
//! deliberately simple interval analysis of a generated program. It keeps
//! one interval per name, runs loops trip by trip and calls into their
//! bodies, joins at every merge, and records the interval of the value at
//! every site (the same sites, kinds and nodes as check_gen::solve).
//!
//! A proven claim whose value this analysis already keeps inside the claim
//! is justified; one it cannot justify is flagged, and the solver decides.
//! Written separately from the solver translation on purpose: two ways of
//! reading the same program, neither shared with CCA.

use crate::ast::*;
use std::collections::HashMap;

/// The interval of every site's value, keyed by (node, statement, kind),
/// joined over every time the site is reached.
pub type SiteRanges = HashMap<(usize, usize, &'static str), (i128, i128)>;

const TRIPS: usize = 14;
const DEPTH: usize = 12;
const LIMIT_LO: i128 = -(1i128 << 63);
const LIMIT_HI: i128 = (1i128 << 64) - 1;
/// Far enough from i128's ends that sums and small products cannot overflow it.
const BIG: i128 = 1i128 << 100;

#[derive(Clone, Copy, Debug, PartialEq)]
struct R(i128, i128);

impl R {
    fn point(v: i128) -> R {
        R(v, v)
    }
    fn join(self, o: R) -> R {
        R(self.0.min(o.0), self.1.max(o.1))
    }
    fn meet(self, o: R) -> Option<R> {
        let r = R(self.0.max(o.0), self.1.min(o.1));
        (r.0 <= r.1).then_some(r)
    }
    fn clamp(self) -> R {
        R(self.0.clamp(-BIG, BIG), self.1.clamp(-BIG, BIG))
    }
    const TOP: R = R(-BIG, BIG);
    const BOOL: R = R(0, 1);
}

fn range_of(t: TyR) -> R {
    let (lo, hi) = if t.base == Ty::Bool {
        (0, 1)
    } else if t.base.signed() {
        (-(1i128 << (t.base.bits() - 1)), (1i128 << (t.base.bits() - 1)) - 1)
    } else {
        (0, (1i128 << t.base.bits()) - 1)
    };
    match t.range {
        None => R(lo, hi),
        Some((a, b)) => R(a.unwrap_or(lo), b.unwrap_or(hi)),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct V {
    r: R,
    ty: TyR,
}

type Env = Vec<HashMap<String, V>>;

fn join_env(a: Option<Env>, b: Option<Env>) -> Option<Env> {
    match (a, b) {
        (None, x) | (x, None) => x,
        (Some(mut a), Some(b)) => {
            for (sa, sb) in a.iter_mut().zip(b.iter()) {
                for (k, v) in sa.iter_mut() {
                    if let Some(w) = sb.get(k) {
                        v.r = v.r.join(w.r);
                    }
                }
            }
            Some(a)
        }
    }
}

struct An<'a> {
    p: &'a Program,
    out: SiteRanges,
    node: usize,
    stmt: usize,
    call_at: usize,
    depth: usize,
    /// per enclosing loop: break states, continue states
    loops: Vec<(Vec<Env>, Vec<Env>)>,
    /// per enclosing function: return values and the return type
    rets: Vec<(Option<R>, TyR)>,
    /// Some body was not followed (a loop past its trips, a call past the
    /// depth): the recorded ranges may miss values, so they justify nothing.
    incomplete: bool,
}

fn corners(a: R, b: R, f: impl Fn(i128, i128) -> i128) -> R {
    let v = [f(a.0, b.0), f(a.0, b.1), f(a.1, b.0), f(a.1, b.1)];
    R(*v.iter().min().unwrap(), *v.iter().max().unwrap()).clamp()
}

fn floor_div(a: i128, b: i128) -> i128 {
    let q = a / b;
    if a % b != 0 && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

impl<'a> An<'a> {
    fn record(&mut self, kind: &'static str, r: R) {
        let key = (self.node, self.stmt, kind);
        let e = self.out.entry(key).or_insert((r.0, r.1));
        *e = (e.0.min(r.0), e.1.max(r.1));
    }

    fn lookup<'e>(&self, env: &'e Env, n: &str) -> Option<&'e V> {
        env.iter().rev().find_map(|s| s.get(n))
    }

    /// A value landing in `t`: recorded, then cut to what `t` holds (past
    /// that the program has stopped).
    fn land(&mut self, r: R, t: TyR) -> R {
        if t.base == Ty::Bool {
            return R::BOOL;
        }
        self.record("narrowing", r);
        r.meet(range_of(t)).unwrap_or(range_of(t))
    }

    fn limit64(&mut self, r: R) -> R {
        self.record("past_64_bits", r);
        r.meet(R(LIMIT_LO, LIMIT_HI)).unwrap_or(R(LIMIT_LO, LIMIT_HI))
    }

    fn ty_of(&self, env: &Env, e: &Expr) -> Option<Ty> {
        match e {
            Expr::Var(n) => self.lookup(env, n).map(|v| v.ty.base),
            Expr::To(t, _) => Some(*t),
            Expr::Call(f, _) => self.p.funcs.iter().find(|g| &g.name == f).map(|g| g.ret.base),
            Expr::Bin(_, a, b) => self.ty_of(env, a).or_else(|| self.ty_of(env, b)),
            Expr::Neg(a) | Expr::NotBit(a) => self.ty_of(env, a),
            _ => None,
        }
    }

    /// Booleans: R(0,0) certainly false, R(1,1) certainly true, R(0,1) either.
    fn expr(&mut self, env: &mut Env, e: &Expr) -> R {
        // Bare numbers alone: their exact answer, no sites in between.
        if !matches!(e, Expr::Lit(_)) {
            if let Some(v) = fold_bare(e) {
                return R::point(v.clamp(-BIG, BIG));
            }
        }
        match e {
            // Lists are not analysed yet: a program with one has no complete answer.
            Expr::At(..) | Expr::Len(_) | Expr::Lend(_) | Expr::Case(..) | Expr::Match(..) | Expr::SimdFill(_) | Expr::SimdFold(..) | Expr::ToEnum(..) | Expr::Num(_) | Expr::Text(_) | Expr::Conv(..) | Expr::StrBytes(_) | Expr::Src(_) => {
                self.incomplete = true;
                R::TOP
            }
            Expr::Lit(v) => R::point(*v),
            Expr::Bool(b) => R::point(*b as i128),
            Expr::Var(n) => self.lookup(env, n).map(|v| v.r).unwrap_or(R::TOP),
            Expr::Neg(a) => {
                let x = self.expr(env, a);
                self.node = addr(e);
                self.limit64(R(-x.1, -x.0))
            }
            Expr::Not(a) => {
                let x = self.expr(env, a);
                R(1 - x.1, 1 - x.0)
            }
            // `and.then` / `or.else`: as `and` / `or` here; the right side's
            // values are over-approximated (it may not run), which is safe.
            Expr::And(a, b) | Expr::AndThen(a, b) => {
                let (x, y) = (self.expr(env, a), self.expr(env, b));
                R(x.0.min(y.0), x.1.min(y.1))
            }
            Expr::Or(a, b) | Expr::OrElse(a, b) => {
                let (x, y) = (self.expr(env, a), self.expr(env, b));
                R(x.0.max(y.0), x.1.max(y.1))
            }
            Expr::Cmp(op, a, b) => {
                let (x, y) = (self.expr(env, a), self.expr(env, b));
                let (sure, never) = match op {
                    CmpOp::Eq => (x.0 == x.1 && y.0 == y.1 && x.0 == y.0, x.1 < y.0 || y.1 < x.0),
                    CmpOp::Ne => (x.1 < y.0 || y.1 < x.0, x.0 == x.1 && y.0 == y.1 && x.0 == y.0),
                    CmpOp::Lt => (x.1 < y.0, x.0 >= y.1),
                    CmpOp::Gt => (x.0 > y.1, x.1 <= y.0),
                    CmpOp::Le => (x.1 <= y.0, x.0 > y.1),
                    CmpOp::Ge => (x.0 >= y.1, x.1 < y.0),
                };
                if sure {
                    R::point(1)
                } else if never {
                    R::point(0)
                } else {
                    R::BOOL
                }
            }
            Expr::To(t, a) => {
                let x = self.expr(env, a);
                self.node = addr(e);
                self.stmt = 0;
                self.land(x, (*t).into())
            }
            Expr::NotBit(a) => {
                let t = self.ty_of(env, a).unwrap_or(Ty::I64);
                let x = self.expr(env, a);
                if t.signed() {
                    R(-x.1 - 1, -x.0 - 1)
                } else {
                    let m = range_of(t.into()).1;
                    R(m - x.1, m - x.0)
                }
            }
            Expr::Call(f, args) => {
                let saved = std::mem::replace(&mut self.call_at, addr(e));
                let r = self.call(env, f, args);
                self.call_at = saved;
                r
            }
            Expr::Bin(op, a, b) => {
                let t = self.ty_of(env, e).unwrap_or(Ty::I64);
                let x = self.expr(env, a);
                let y = self.expr(env, b);
                self.node = addr(e);
                self.binary(*op, x, y, t)
            }
        }
    }

    fn binary(&mut self, op: BinOp, x: R, y: R, t: Ty) -> R {
        use BinOp::*;
        let tr = range_of(t.into());
        let w = t.bits() as i128;
        match op {
            Add => self.limit64(R(x.0 + y.0, x.1 + y.1).clamp()),
            Sub => self.limit64(R(x.0 - y.1, x.1 - y.0).clamp()),
            Mul => {
                let r = corners(x.clamp_small(), y.clamp_small(), |a, b| a.saturating_mul(b));
                self.limit64(r)
            }
            Div | Mod => {
                self.record(if op == Div { "division_by_zero" } else { "modulo_by_zero" }, y);
                // past the site the divisor is not zero
                let parts: Vec<R> = [R(y.0, y.1.min(-1)), R(y.0.max(1), y.1)].into_iter().filter(|r| r.0 <= r.1).collect();
                if parts.is_empty() {
                    return R::TOP;
                }
                let mut out: Option<R> = None;
                for d in parts {
                    let r = if op == Div {
                        corners(x, d, floor_div)
                    } else if d.0 > 0 {
                        if x.0 >= 0 && x.1 < d.0 {
                            x
                        } else {
                            R(0, d.1 - 1)
                        }
                    } else if x.1 <= 0 && x.0 > d.1 {
                        x
                    } else {
                        R(d.0 + 1, 0)
                    };
                    out = Some(out.map_or(r, |o| o.join(r)));
                }
                let r = out.unwrap();
                if op == Div {
                    self.limit64(r)
                } else {
                    r
                }
            }
            Pow => {
                self.record("negative_exponent", y);
                let e = R(y.0.max(0), y.1);
                if e.0 > e.1 {
                    return R::TOP;
                }
                if x.1 - x.0 <= 64 && e.1 - e.0 <= 64 && e.1 <= 200 {
                    let mut lo = i128::MAX;
                    let mut hi = i128::MIN;
                    for b in x.0..=x.1 {
                        for k in e.0..=e.1 {
                            let v = b.checked_pow(k as u32).map(|v| v.clamp(-BIG, BIG)).unwrap_or(if b < 0 && k % 2 == 1 { -BIG } else { BIG });
                            lo = lo.min(v);
                            hi = hi.max(v);
                        }
                    }
                    self.limit64(R(lo, hi))
                } else {
                    self.limit64(R::TOP)
                }
            }
            AndBit | OrBit | Xor => {
                if x.0 >= 0 && y.0 >= 0 {
                    let top = x.1.max(y.1);
                    let mask = if top == 0 { 0 } else { (1i128 << (128 - top.leading_zeros())) - 1 };
                    if op == AndBit {
                        R(0, x.1.min(y.1))
                    } else {
                        R(0, mask)
                    }
                } else {
                    tr
                }
            }
            Shl | Shr => {
                self.record("shift_count", y);
                let k = R(y.0.max(0), y.1.min(w - 1));
                if k.0 > k.1 {
                    return tr;
                }
                if op == Shl {
                    let r = corners(x.clamp_small(), k, |a, s| a.saturating_mul(1i128 << s));
                    self.record("shift_width", r);
                    r.meet(tr).unwrap_or(tr)
                } else if x.0 >= 0 {
                    R(x.0 >> k.1, x.1 >> k.0)
                } else {
                    R(x.0.min(0), x.1.max(0))
                }
            }
            WAdd | WSub | WMul => {
                let exact = match op {
                    WAdd => R(x.0 + y.0, x.1 + y.1).clamp(),
                    WSub => R(x.0 - y.1, x.1 - y.0).clamp(),
                    _ => corners(x.clamp_small(), y.clamp_small(), |a, b| a.saturating_mul(b)),
                };
                if exact.0 >= tr.0 && exact.1 <= tr.1 {
                    exact
                } else {
                    tr
                }
            }
        }
    }

    fn call(&mut self, env: &mut Env, f: &str, args: &[Expr]) -> R {
        if f == "put_i" || f == "put_u" {
            let t = if f == "put_i" { Ty::I64 } else { Ty::U64 };
            let x = self.expr(env, &args[0]);
            (self.node, self.stmt) = (addr(&args[0]), self.call_at);
            self.land(x, t.into());
            return R::point(0);
        }
        let Some(func) = self.p.funcs.iter().find(|g| g.name == f) else { return R::TOP };
        let mut vals = vec![];
        for (a, prm) in args.iter().zip(&func.params) {
            let x = self.expr(env, a);
            (self.node, self.stmt) = (addr(a), self.call_at);
            vals.push(self.land(x, prm.ty));
        }
        if self.depth >= DEPTH {
            self.incomplete = true;
            return range_of(func.ret);
        }
        self.depth += 1;
        let mut fenv: Env = vec![HashMap::new()];
        for (prm, v) in func.params.iter().zip(vals) {
            fenv[0].insert(prm.name.clone(), V { r: v, ty: prm.ty });
        }
        self.rets.push((None, func.ret));
        let _ = self.block(Some(fenv), &func.body);
        let (r, _) = self.rets.pop().unwrap();
        self.depth -= 1;
        r.unwrap_or(range_of(func.ret))
    }

    fn block(&mut self, env: Option<Env>, body: &[Stmt]) -> Option<Env> {
        let mut env = env?;
        env.push(HashMap::new());
        let mut cur = Some(env);
        for s in body {
            let Some(e) = cur else { break };
            cur = self.stmt(e, s);
        }
        cur.map(|mut e| {
            e.pop();
            e
        })
    }

    fn set(env: &mut Env, n: &str, r: R) {
        for s in env.iter_mut().rev() {
            if let Some(v) = s.get_mut(n) {
                v.r = r;
                return;
            }
        }
    }

    fn widen_to_types(env: &mut Env) {
        for s in env.iter_mut() {
            for v in s.values_mut() {
                v.r = range_of(v.ty);
            }
        }
    }

    fn stmt(&mut self, mut env: Env, s: &Stmt) -> Option<Env> {
        match s {
            Stmt::ListLet { .. } | Stmt::SetAt { .. } | Stmt::Append { .. } | Stmt::ForEach { .. } | Stmt::Borrow { .. } | Stmt::Give { .. } | Stmt::EnumLet { .. } | Stmt::MatchStmt { .. } | Stmt::SimdLet { .. } | Stmt::TypedLet { .. } | Stmt::AutoLet { .. } | Stmt::ListFill { .. } => {
                self.incomplete = true;
                Some(env)
            }
            Stmt::Let { name, ty, value, .. } => {
                let x = self.expr(&mut env, value);
                (self.node, self.stmt) = (addr(value), addr(s));
                let r = self.land(x, *ty);
                env.last_mut().unwrap().insert(name.clone(), V { r, ty: *ty });
                Some(env)
            }
            Stmt::Assign { name, value, .. } => {
                let ty = self.lookup(&env, name).map(|v| v.ty).unwrap_or(Ty::I64.into());
                let x = self.expr(&mut env, value);
                (self.node, self.stmt) = (addr(value), addr(s));
                let r = self.land(x, ty);
                Self::set(&mut env, name, r);
                Some(env)
            }
            Stmt::Print { signed, value } => {
                let t = if *signed { Ty::I64 } else { Ty::U64 };
                self.call_at = addr(s);
                let x = self.expr(&mut env, value);
                (self.node, self.stmt) = (addr(value), addr(s));
                self.land(x, t.into());
                Some(env)
            }
            Stmt::Return(e) => {
                let x = self.expr(&mut env, e);
                (self.node, self.stmt) = (addr(e), addr(s));
                if let Some((acc, rt)) = self.rets.last().cloned() {
                    let r = self.land(x, rt);
                    self.rets.last_mut().unwrap().0 = Some(acc.map_or(r, |a| a.join(r)));
                }
                None
            }
            Stmt::Break => {
                if let Some(l) = self.loops.last_mut() {
                    l.0.push(env);
                }
                None
            }
            Stmt::Continue => {
                if let Some(l) = self.loops.last_mut() {
                    l.1.push(env);
                }
                None
            }
            Stmt::Block { body, .. } => self.block(Some(env), body),
            Stmt::If { arms, otherwise } => {
                let mut out: Option<Env> = None;
                let mut rest = Some(env);
                for (cond, body) in arms {
                    let Some(mut e) = rest.take() else { break };
                    let c = self.expr(&mut e, cond);
                    if c.1 == 1 {
                        out = join_env(out, self.block(Some(e.clone()), body));
                    }
                    if c.0 == 0 {
                        rest = Some(e);
                    }
                }
                match otherwise {
                    Some(body) => join_env(out, self.block(rest, body)),
                    None => join_env(out, rest),
                }
            }
            Stmt::For { perm, name, ty, start, stop, step, body } => {
                let (Expr::Lit(a), Expr::Lit(b)) = (start, stop) else { return Some(env) };
                let st = step.unwrap_or(1);
                let mut vals = vec![];
                let mut i = *a;
                while (st > 0 && i <= *b) || (st < 0 && i >= *b) {
                    vals.push(i);
                    i += st;
                    if vals.len() > TRIPS {
                        break;
                    }
                }
                let too_many = vals.len() > TRIPS;
                self.loops.push((vec![], vec![]));
                let mut cur = Some(env);
                for v in vals.iter().take(TRIPS) {
                    let Some(mut e) = cur.take() else { break };
                    let mut scope = HashMap::new();
                    scope.insert(name.clone(), V { r: R::point(*v), ty: (*ty).into() });
                    e.push(scope);
                    let after = self.block(Some(e), body).map(|mut e| {
                        e.pop();
                        e
                    });
                    let conts: Vec<Env> = std::mem::take(&mut self.loops.last_mut().unwrap().1)
                        .into_iter()
                        .map(|mut e| {
                            e.pop();
                            e
                        })
                        .collect();
                    cur = conts.into_iter().fold(after, |a, c| join_env(a, Some(c)));
                }
                let (brks, _) = self.loops.pop().unwrap();
                let mut out = brks.into_iter().map(|mut e| {
                    e.pop();
                    e
                }).fold(cur, |a, b| join_env(a, Some(b)));
                if too_many {
                    self.incomplete = true;
                    if let Some(e) = out.as_mut() {
                        Self::widen_to_types(e);
                    }
                }
                if *perm {
                    if let Some(e) = out.as_mut() {
                        let lo = vals.iter().copied().min().unwrap_or(*a).min(*a);
                        let hi = vals.iter().copied().max().unwrap_or(*a).max(*a);
                        e.last_mut().unwrap().insert(name.clone(), V { r: R(lo, hi), ty: (*ty).into() });
                    }
                }
                out
            }
            Stmt::While { cond, body } => self.run_loop(env, Some(cond), body),
            Stmt::Loop { body } => self.run_loop(env, None, body),
        }
    }

    fn run_loop(&mut self, env: Env, cond: Option<&Expr>, body: &[Stmt]) -> Option<Env> {
        self.loops.push((vec![], vec![]));
        let mut exits: Option<Env> = None;
        let mut cur = Some(env);
        let mut trips = 0;
        while let Some(mut e) = cur.take() {
            if trips == TRIPS {
                self.incomplete = true;
                // Past what we follow: every name may hold anything its type does.
                let mut w = e;
                Self::widen_to_types(&mut w);
                exits = join_env(exits, Some(w));
                break;
            }
            trips += 1;
            if let Some(c) = cond {
                let r = self.expr(&mut e, c);
                if r.0 == 0 {
                    exits = join_env(exits, Some(e.clone()));
                }
                if r.1 == 0 {
                    break;
                }
            }
            let after = self.block(Some(e), body);
            let conts = std::mem::take(&mut self.loops.last_mut().unwrap().1);
            cur = conts.into_iter().fold(after, |a, c| join_env(a, Some(c)));
        }
        let (brks, _) = self.loops.pop().unwrap();
        brks.into_iter().fold(exits, |a, b| join_env(a, Some(b)))
    }
}

impl R {
    /// Keep products of two intervals inside i128.
    fn clamp_small(self) -> R {
        R(self.0.clamp(-(1i128 << 62) * 4, (1i128 << 62) * 4), self.1.clamp(-(1i128 << 62) * 4, (1i128 << 62) * 4))
    }
}

/// Every site's value interval, for inputs in 0..=input_max; `None` when
/// the analysis could not follow the whole program.
pub fn analyse(p: &Program, input_max: u128) -> Option<SiteRanges> {
    let mut a = An { p, out: HashMap::new(), node: 0, stmt: 0, call_at: 0, depth: 0, loops: vec![], rets: vec![], incomplete: false };
    let mut env: Env = vec![HashMap::new()];
    for i in &p.inputs {
        env[0].insert(i.name.clone(), V { r: R(0, input_max as i128), ty: Ty::U64.into() });
    }
    let _ = a.block(Some(env), &p.main);
    (!a.incomplete).then_some(a.out)
}
