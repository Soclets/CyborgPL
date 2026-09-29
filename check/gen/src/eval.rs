//! An exact evaluator of a generated program on one input: every value
//! every site sees, until the program stops. It confirms a claim the solver
//! refuted (attack (c)): the solver's translation approximates on purpose
//! (loops cut at a bound, results past a depth left unknown), so a refutation
//! only counts when this evaluator, running the actual program on the
//! solver's input, sees the value outside the claim or the site fail.
//!
//! Semantics as design/ says and both engines do today: exact integer maths
//! checked where it lands and against the 64-bit window in between; bare
//! numbers alone folded exactly; `/` floors, `mod` takes the divisor's sign;
//! bitwise at the operands' width; shift counts 0..width-1; both sides of
//! `and` / `or` evaluated, left then right (owner, fc04c70).

use crate::ast::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Values seen per site, keyed like check_gen::intervals (node, stmt, kind).
pub type Seen = HashMap<(usize, usize, &'static str), Vec<i128>>;

pub struct Run {
    pub seen: Seen,
    /// The site the program stopped at, if it did.
    pub stopped: Option<(usize, usize, &'static str)>,
    /// Ran out of steps before the end (so `seen` is partial).
    pub exhausted: bool,
}

const LIMIT_LO: i128 = -(1i128 << 63);
const LIMIT_HI: i128 = (1i128 << 64) - 1;
const MAX_STEPS: u64 = 5_000_000;
const MAX_DEPTH: usize = 64;

#[derive(Clone)]
enum Val {
    I(i128),
    B(bool),
    /// A list; a lend of it shares the storage.
    L(Rc<RefCell<Vec<i128>>>),
    /// An enum value: its case's index and data (0 if none).
    E(usize, i128),
    /// A simd vector's lanes; one lane (from `simd.fill`) stands for all.
    S(Vec<i128>),
}

impl Val {
    fn i(self) -> i128 {
        match self {
            Val::I(v) => v,
            Val::B(b) => b as i128,
            Val::L(_) | Val::E(..) | Val::S(_) => 0,
        }
    }
    fn b(self) -> bool {
        match self {
            Val::B(b) => b,
            Val::I(v) => v != 0,
            Val::L(_) | Val::E(..) | Val::S(_) => false,
        }
    }
}

enum Flow {
    Next,
    Break,
    Continue,
    Return(Val),
}

/// The program has stopped: at this site. Or, `Leave`, a `match` arm in an
/// expression has left (`return`, `break`, `continue`): the statement around
/// it turns that back into its flow.
enum Stop {
    At((usize, usize, &'static str)),
    Leave(Flow),
}

#[allow(non_snake_case)]
fn Stop(k: (usize, usize, &'static str)) -> Stop {
    Stop::At(k)
}

struct Ev<'a> {
    p: &'a Program,
    seen: Seen,
    node: usize,
    stmt: usize,
    call_at: usize,
    steps: u64,
    depth: usize,
    /// The return type of each function being run.
    ret_ty: Vec<TyR>,
}

fn range_of(t: TyR) -> (i128, i128) {
    let (lo, hi) = if t.base == Ty::Bool {
        (0, 1)
    } else if t.base.signed() {
        (-(1i128 << (t.base.bits() - 1)), (1i128 << (t.base.bits() - 1)) - 1)
    } else {
        (0, (1i128 << t.base.bits()) - 1)
    };
    match t.range {
        None => (lo, hi),
        Some((a, b)) => (a.unwrap_or(lo), b.unwrap_or(hi)),
    }
}

type Env = Vec<HashMap<String, (Val, TyR)>>;

impl<'a> Ev<'a> {
    fn tick(&mut self) -> Result<(), Stop> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return Err(Stop((0, 0, "steps")));
        }
        Ok(())
    }

    fn see(&mut self, kind: &'static str, v: i128) {
        let e = self.seen.entry((self.node, self.stmt, kind)).or_default();
        if e.len() < 256 && !e.contains(&v) {
            e.push(v);
        }
    }

    fn check(&mut self, kind: &'static str, v: i128, ok: bool) -> Result<(), Stop> {
        self.see(kind, v);
        if ok {
            Ok(())
        } else {
            Err(Stop((self.node, self.stmt, kind)))
        }
    }

    fn land(&mut self, v: i128, t: TyR) -> Result<i128, Stop> {
        if t.base == Ty::Bool {
            return Ok(v);
        }
        let (lo, hi) = range_of(t);
        self.check("narrowing", v, lo <= v && v <= hi)?;
        Ok(v)
    }

    fn limit64(&mut self, v: i128) -> Result<i128, Stop> {
        self.check("past_64_bits", v, (LIMIT_LO..=LIMIT_HI).contains(&v))?;
        Ok(v)
    }

    fn lookup(env: &Env, n: &str) -> Option<(Val, TyR)> {
        env.iter().rev().find_map(|s| s.get(n).cloned())
    }

    fn ty_of(&self, env: &Env, e: &Expr) -> Option<Ty> {
        match e {
            Expr::Var(n) => Self::lookup(env, n).map(|v| v.1.base),
            Expr::To(t, _) => Some(*t),
            Expr::SimdFill(a) => self.ty_of(env, a),
            Expr::At(n, _) => Self::lookup(env, n).map(|v| v.1.base),
            Expr::Len(_) => Some(Ty::U64),
            Expr::Call(f, _) => self.p.funcs.iter().find(|g| &g.name == f).map(|g| g.ret.base),
            Expr::Bin(_, a, b) => self.ty_of(env, a).or_else(|| self.ty_of(env, b)),
            Expr::Neg(a) | Expr::NotBit(a) => self.ty_of(env, a),
            _ => None,
        }
    }

    fn expr(&mut self, env: &mut Env, e: &Expr) -> Result<Val, Stop> {
        self.tick()?;
        if !matches!(e, Expr::Lit(_)) {
            if let Some(v) = fold_bare(e) {
                return Ok(Val::I(v));
            }
        }
        Ok(match e {
            Expr::Lit(v) => Val::I(*v),
            Expr::Bool(b) => Val::B(*b),
            Expr::Var(n) => Self::lookup(env, n).map(|v| v.0).unwrap_or(Val::I(0)),
            Expr::Neg(a) => {
                let x = self.expr(env, a)?.i();
                self.node = addr(e);
                Val::I(self.limit64(-x)?)
            }
            Expr::Not(a) => Val::B(!self.expr(env, a)?.b()),
            Expr::And(a, b) => {
                let x = self.expr(env, a)?.b();
                let y = self.expr(env, b)?.b();
                Val::B(x && y)
            }
            Expr::Or(a, b) => {
                let x = self.expr(env, a)?.b();
                let y = self.expr(env, b)?.b();
                Val::B(x || y)
            }
            Expr::AndThen(a, b) => Val::B(self.expr(env, a)?.b() && self.expr(env, b)?.b()),
            Expr::OrElse(a, b) => Val::B(self.expr(env, a)?.b() || self.expr(env, b)?.b()),
            Expr::Cmp(op, a, b) => {
                let (x, y) = (self.expr(env, a)?, self.expr(env, b)?);
                if let (Val::E(c, d), Val::E(c2, d2)) = (&x, &y) {
                    let same = c == c2 && d == d2;
                    return Ok(Val::B(if *op == CmpOp::Eq { same } else { !same }));
                }
                if matches!(x, Val::S(_)) || matches!(y, Val::S(_)) {
                    // One answer per lane (simd.md), kept as 0 / 1.
                    let (xs, ys) = (lanes(&x), lanes(&y));
                    let n = xs.len().max(ys.len());
                    let out = (0..n).map(|i| cmp(*op, xs[i.min(xs.len() - 1)], ys[i.min(ys.len() - 1)]) as i128).collect();
                    return Ok(Val::S(out));
                }
                let (x, y) = (x.i(), y.i());
                Val::B(match op {
                    CmpOp::Eq => x == y,
                    CmpOp::Ne => x != y,
                    CmpOp::Lt => x < y,
                    CmpOp::Gt => x > y,
                    CmpOp::Le => x <= y,
                    CmpOp::Ge => x >= y,
                })
            }
            Expr::To(t, a) => {
                let x = self.expr(env, a)?.i();
                self.node = addr(e);
                self.stmt = 0;
                Val::I(self.land(x, (*t).into())?)
            }
            Expr::NotBit(a) => {
                let t = self.ty_of(env, a).unwrap_or(Ty::I64);
                let x = self.expr(env, a)?.i();
                Val::I(if t.signed() { -x - 1 } else { range_of(t.into()).1 - x })
            }
            Expr::Call(f, args) => {
                let saved = std::mem::replace(&mut self.call_at, addr(e));
                let r = self.call(env, f, args);
                self.call_at = saved;
                r?
            }
            Expr::At(n, p) if matches!(Self::lookup(env, n), Some((Val::S(_), _))) => {
                let x = self.expr(env, p)?.i();
                let Some((Val::S(v), _)) = Self::lookup(env, n) else { unreachable!() };
                (self.node, self.stmt) = (addr(e), 0);
                self.check("position", x, 1 <= x && x <= v.len() as i128)?;
                Val::I(v[(x - 1) as usize])
            }
            Expr::At(n, p) => {
                let x = self.expr(env, p)?.i();
                let l = Self::list(env, n);
                let len = l.borrow().len() as i128;
                (self.node, self.stmt) = (addr(e), 0);
                self.check("position", x, 1 <= x && x <= len)?;
                let v = l.borrow()[(x - 1) as usize];
                Val::I(v)
            }
            Expr::Len(n) => Val::I(Self::list(env, n).borrow().len() as i128),
            Expr::Lend(n) => Self::lookup(env, n).map(|v| v.0).unwrap_or(Val::I(0)),
            Expr::ToEnum(en, x, fallback) => {
                let v = self.expr(env, x)?.i();
                let vals = self.p.enums.iter().find(|d| &d.name == en).and_then(|d| d.values.clone()).unwrap_or_default();
                match vals.iter().position(|w| *w == v) {
                    Some(i) => Val::E(i, 0),
                    None => self.expr(env, fallback)?,
                }
            }
            Expr::Case(en, c, data) => {
                let (idx, ty) = self.case(en, c);
                let d = match data {
                    Some(d) => {
                        let x = self.expr(env, d)?.i();
                        (self.node, self.stmt) = (addr(&**d), addr(e));
                        self.land(x, ty.unwrap_or(Ty::I64).into())?
                    }
                    None => 0,
                };
                Val::E(idx, d)
            }
            Expr::Match(scrut, arms) => {
                let x = self.expr(env, scrut)?;
                match self.arm(env, &x, arms)? {
                    Ok(v) => v,
                    // An arm that leaves: carried out through the statement.
                    Err(f) => return Err(Stop::Leave(f)),
                }
            }
            Expr::Bin(op, a, b) => {
                let t = self.ty_of(env, e).unwrap_or(Ty::I64);
                let x = self.expr(env, a)?;
                let y = self.expr(env, b)?;
                self.node = addr(e);
                if matches!(x, Val::S(_)) || matches!(y, Val::S(_)) {
                    // Lane by lane, each by the plain rule.
                    let (xs, ys) = (lanes(&x), lanes(&y));
                    let n = xs.len().max(ys.len());
                    let mut out = vec![];
                    for i in 0..n {
                        let v = self.binary(*op, xs[i.min(xs.len() - 1)], ys[i.min(ys.len() - 1)], t)?;
                        // Each lane `+ - x xx` is checked at the lane's type
                        // (owner: b, fc1cea9); the wrapping forms are not.
                        let v = if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow) { self.land(v, t.into())? } else { v };
                        out.push(v);
                    }
                    Val::S(out)
                } else {
                    Val::I(self.binary(*op, x.i(), y.i(), t)?)
                }
            }
            Expr::SimdFill(a) => Val::S(vec![self.expr(env, a)?.i()]),
            Expr::SimdFold(f, a) => {
                let t = self.ty_of(env, a).unwrap_or(Ty::I64);
                let xs = lanes(&self.expr(env, a)?);
                match *f {
                    "max" => Val::I(*xs.iter().max().unwrap_or(&0)),
                    "min" => Val::I(*xs.iter().min().unwrap_or(&0)),
                    "all" => Val::B(xs.iter().all(|x| *x != 0)),
                    "any" => Val::B(xs.iter().any(|x| *x != 0)),
                    _ => {
                        // sum: a check site, it can leave the lane type
                        (self.node, self.stmt) = (addr(e), 0);
                        Val::I(self.land(xs.iter().sum(), t.into())?)
                    }
                }
            }
        })
    }

    fn wrap(v: i128, t: Ty) -> i128 {
        let bits = t.bits();
        let m = v & ((1i128 << bits) - 1);
        if t.signed() && m >= 1i128 << (bits - 1) {
            m - (1i128 << bits)
        } else {
            m
        }
    }

    fn binary(&mut self, op: BinOp, x: i128, y: i128, t: Ty) -> Result<i128, Stop> {
        use BinOp::*;
        let w = t.bits() as i128;
        let (tlo, thi) = range_of(t.into());
        match op {
            Add => self.limit64(x + y),
            Sub => self.limit64(x - y),
            Mul => {
                let r = x.checked_mul(y).unwrap_or(i128::MAX);
                self.limit64(r)
            }
            Div | Mod => {
                self.check(if op == Div { "division_by_zero" } else { "modulo_by_zero" }, y, y != 0)?;
                let q = floor_div(x, y);
                if op == Div {
                    self.limit64(q)
                } else {
                    Ok(x - q * y)
                }
            }
            Pow => {
                self.check("negative_exponent", y, y >= 0)?;
                let r = if y > 200 { if x == 0 || x == 1 { x } else if x == -1 { if y % 2 == 0 { 1 } else { -1 } } else { i128::MAX } } else { x.checked_pow(y as u32).unwrap_or(i128::MAX) };
                self.limit64(r)
            }
            AndBit => Ok(x & y),
            OrBit => Ok(x | y),
            Xor => Ok(x ^ y),
            Shl | Shr => {
                self.check("shift_count", y, 0 <= y && y < w)?;
                if op == Shl {
                    let r = x.checked_mul(1i128 << y).unwrap_or(i128::MAX);
                    self.check("shift_width", r, tlo <= r && r <= thi)?;
                    Ok(r)
                } else {
                    Ok(x >> y)
                }
            }
            WAdd => Ok(Self::wrap(x + y, t)),
            WSub => Ok(Self::wrap(x - y, t)),
            WMul => Ok(Self::wrap(x.wrapping_mul(y), t)),
        }
    }

    fn call(&mut self, env: &mut Env, f: &str, args: &[Expr]) -> Result<Val, Stop> {
        if f == "put_i" || f == "put_u" {
            let t = if f == "put_i" { Ty::I64 } else { Ty::U64 };
            let x = self.expr(env, &args[0])?.i();
            (self.node, self.stmt) = (addr(&args[0]), self.call_at);
            self.land(x, t.into())?;
            return Ok(Val::I(0));
        }
        if let Some(bf) = self.p.bfuncs.iter().find(|g| g.name == f) {
            // A borrow function: its lists come in lent and a borrow goes out.
            let mut fenv: Env = vec![HashMap::new()];
            for (a, (n, _)) in args.iter().zip(&bf.params) {
                let x = self.expr(env, a)?;
                fenv[0].insert(n.clone(), (x, bf.elem.into()));
            }
            if self.depth >= MAX_DEPTH {
                return Err(Stop((0, 0, "steps")));
            }
            self.depth += 1;
            let r = self.block(&mut fenv, &bf.body);
            self.depth -= 1;
            return match r? {
                Flow::Return(v) => Ok(v),
                _ => Ok(Val::I(0)),
            };
        }
        let Some(func) = self.p.funcs.iter().find(|g| g.name == f) else { return Ok(Val::I(0)) };
        let mut fenv: Env = vec![HashMap::new()];
        for (a, prm) in args.iter().zip(&func.params) {
            let x = self.expr(env, a)?;
            (self.node, self.stmt) = (addr(a), self.call_at);
            let v = if prm.ty.base == Ty::Bool { x } else { Val::I(self.land(x.i(), prm.ty)?) };
            fenv[0].insert(prm.name.clone(), (v, prm.ty));
        }
        if self.depth >= MAX_DEPTH {
            return Err(Stop((0, 0, "steps")));
        }
        self.depth += 1;
        self.ret_ty.push(func.ret);
        let r = self.block(&mut fenv, &func.body);
        self.ret_ty.pop();
        self.depth -= 1;
        match r? {
            Flow::Return(v) => Ok(v),
            _ => Ok(Val::I(0)),
        }
    }

    fn block(&mut self, env: &mut Env, body: &[Stmt]) -> Result<Flow, Stop> {
        env.push(HashMap::new());
        let mut out = Flow::Next;
        for s in body {
            match self.stmt(env, s) {
                Ok(Flow::Next) => {}
                Ok(f) => {
                    out = f;
                    break;
                }
                Err(e) => {
                    env.pop();
                    return Err(e);
                }
            }
        }
        env.pop();
        Ok(out)
    }

    fn list(env: &Env, n: &str) -> Rc<RefCell<Vec<i128>>> {
        match Self::lookup(env, n) {
            Some((Val::L(l), _)) => l,
            _ => Rc::new(RefCell::new(vec![])),
        }
    }

    fn case(&self, en: &str, c: &str) -> (usize, Option<Ty>) {
        let d = self.p.enums.iter().find(|d| d.name == en);
        d.and_then(|d| d.cases.iter().position(|x| x.0 == c).map(|i| (i, d.cases[i].1))).unwrap_or((0, None))
    }

    /// Run the arm for `x`: Ok(value) for an arm that gives one, Err(flow)
    /// for one that ran statements.
    fn arm(&mut self, env: &mut Env, x: &Val, arms: &[Arm]) -> Result<Result<Val, Flow>, Stop> {
        let Val::E(c, d) = x else { return Ok(Ok(Val::I(0))) };
        let Some(a) = arms.get(*c) else { return Ok(Ok(Val::I(0))) };
        let mut scope = HashMap::new();
        if let Some(b) = &a.bind {
            let ty = self.p.enums.iter().flat_map(|e| e.cases.iter()).find(|k| k.0 == a.case).and_then(|k| k.1).unwrap_or(Ty::I64);
            scope.insert(b.clone(), (Val::I(*d), ty.into()));
        }
        env.push(scope);
        let r = match &a.value {
            Some(v) => self.expr(env, v).map(Ok),
            None => self.block(env, &a.body).map(Err),
        };
        env.pop();
        r
    }

    fn set(env: &mut Env, n: &str, v: Val) {
        for s in env.iter_mut().rev() {
            if let Some(x) = s.get_mut(n) {
                x.0 = v;
                return;
            }
        }
    }

    fn stmt(&mut self, env: &mut Env, s: &Stmt) -> Result<Flow, Stop> {
        self.tick()?;
        match self.stmt_inner(env, s) {
            // A `match` arm left from inside this statement's expression.
            Err(Stop::Leave(f)) => Ok(f),
            r => r,
        }
    }

    fn stmt_inner(&mut self, env: &mut Env, s: &Stmt) -> Result<Flow, Stop> {
        match s {
            Stmt::SimdLet { name, elem, lanes: n, value, .. } => {
                let x = self.expr(env, value)?;
                let xs = lanes(&x);
                (self.node, self.stmt) = (addr(value), addr(s));
                let mut v = vec![];
                for i in 0..*n {
                    v.push(self.land(xs[i.min(xs.len() - 1)], (*elem).into())?);
                }
                env.last_mut().unwrap().insert(name.clone(), (Val::S(v), (*elem).into()));
                Ok(Flow::Next)
            }
            Stmt::EnumLet { name, value, .. } => {
                let x = self.expr(env, value)?;
                env.last_mut().unwrap().insert(name.clone(), (x, Ty::I64.into()));
                Ok(Flow::Next)
            }
            Stmt::MatchStmt { scrut, arms } => {
                let x = self.expr(env, scrut)?;
                match self.arm(env, &x, arms)? {
                    Ok(_) => Ok(Flow::Next),
                    Err(f) => Ok(f),
                }
            }
            Stmt::Let { name, ty, value, .. } => {
                let x = self.expr(env, value)?;
                (self.node, self.stmt) = (addr(value), addr(s));
                let v = if ty.base == Ty::Bool { x } else { Val::I(self.land(x.i(), *ty)?) };
                env.last_mut().unwrap().insert(name.clone(), (v, *ty));
                Ok(Flow::Next)
            }
            Stmt::Assign { name, value, .. } => {
                let ty = Self::lookup(env, name).map(|v| v.1).unwrap_or(Ty::I64.into());
                let x = self.expr(env, value)?;
                if let Val::E(..) = x {
                    Self::set(env, name, x);
                    return Ok(Flow::Next);
                }
                if let (Val::S(_), Some((Val::S(old), _))) = (&x, Self::lookup(env, name)) {
                    let xs = lanes(&x);
                    (self.node, self.stmt) = (addr(value), addr(s));
                    let mut v = vec![];
                    for i in 0..old.len() {
                        v.push(self.land(xs[i.min(xs.len() - 1)], ty)?);
                    }
                    Self::set(env, name, Val::S(v));
                    return Ok(Flow::Next);
                }
                (self.node, self.stmt) = (addr(value), addr(s));
                let v = if ty.base == Ty::Bool { x } else { Val::I(self.land(x.i(), ty)?) };
                Self::set(env, name, v);
                Ok(Flow::Next)
            }
            Stmt::Print { signed, value } => {
                let t = if *signed { Ty::I64 } else { Ty::U64 };
                self.call_at = addr(s);
                let x = self.expr(env, value)?.i();
                (self.node, self.stmt) = (addr(value), addr(s));
                self.land(x, t.into())?;
                Ok(Flow::Next)
            }
            Stmt::Return(e) => {
                let x = self.expr(env, e)?;
                if let Val::L(_) = x {
                    return Ok(Flow::Return(x));
                }
                (self.node, self.stmt) = (addr(e), addr(s));
                // the enclosing function's return type
                let rt = self.ret_ty.last().copied().unwrap_or(Ty::I64.into());
                let v = if rt.base == Ty::Bool { x } else { Val::I(self.land(x.i(), rt)?) };
                Ok(Flow::Return(v))
            }
            Stmt::Break => Ok(Flow::Break),
            Stmt::Continue => Ok(Flow::Continue),
            Stmt::Block { body, .. } => self.block(env, body),
            Stmt::If { arms, otherwise } => {
                for (c, body) in arms {
                    if self.expr(env, c)?.b() {
                        return self.block(env, body);
                    }
                }
                match otherwise {
                    Some(body) => self.block(env, body),
                    None => Ok(Flow::Next),
                }
            }
            Stmt::For { perm, name, ty, start, stop, step, body } => {
                let (Expr::Lit(a), Expr::Lit(b)) = (start, stop) else { return Ok(Flow::Next) };
                let st = step.unwrap_or(1);
                let mut i = *a;
                let mut last = *a;
                while (st > 0 && i <= *b) || (st < 0 && i >= *b) {
                    last = i;
                    let mut scope = HashMap::new();
                    scope.insert(name.clone(), (Val::I(i), (*ty).into()));
                    env.push(scope);
                    let f = self.block(env, body);
                    env.pop();
                    match f? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        _ => {}
                    }
                    i += st;
                }
                if *perm {
                    env.last_mut().unwrap().insert(name.clone(), (Val::I(last), (*ty).into()));
                }
                Ok(Flow::Next)
            }
            Stmt::While { cond, body } => {
                while self.expr(env, cond)?.b() {
                    match self.block(env, body)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        _ => {}
                    }
                }
                Ok(Flow::Next)
            }
            Stmt::ListLet { name, elem, items, .. } => {
                let mut v = vec![];
                for x in items {
                    let y = self.expr(env, x)?.i();
                    (self.node, self.stmt) = (addr(x), addr(s));
                    v.push(self.land(y, (*elem).into())?);
                }
                env.last_mut().unwrap().insert(name.clone(), (Val::L(Rc::new(RefCell::new(v))), (*elem).into()));
                Ok(Flow::Next)
            }
            Stmt::SetAt { name, pos, value, .. } if matches!(Self::lookup(env, name), Some((Val::S(_), _))) => {
                // One lane of a vector (owner, 8d8a302).
                let Some((Val::S(mut v), ty)) = Self::lookup(env, name) else { unreachable!() };
                let x = self.expr(env, pos)?.i();
                (self.node, self.stmt) = (addr(pos), addr(s));
                self.check("position", x, 1 <= x && x <= v.len() as i128)?;
                let y = self.expr(env, value)?.i();
                (self.node, self.stmt) = (addr(value), addr(s));
                v[(x - 1) as usize] = self.land(y, ty)?;
                Self::set(env, name, Val::S(v));
                Ok(Flow::Next)
            }
            Stmt::SetAt { name, pos, value, .. } => {
                let (l, ty) = match Self::lookup(env, name) {
                    Some((Val::L(l), t)) => (l, t),
                    _ => return Ok(Flow::Next),
                };
                let x = self.expr(env, pos)?.i();
                let len = l.borrow().len() as i128;
                (self.node, self.stmt) = (addr(pos), addr(s));
                self.check("position", x, 1 <= x && x <= len)?;
                let y = self.expr(env, value)?.i();
                (self.node, self.stmt) = (addr(value), addr(s));
                let y = self.land(y, ty)?;
                l.borrow_mut()[(x - 1) as usize] = y;
                Ok(Flow::Next)
            }
            Stmt::Append { name, value } => {
                let (l, ty) = match Self::lookup(env, name) {
                    Some((Val::L(l), t)) => (l, t),
                    _ => return Ok(Flow::Next),
                };
                let y = self.expr(env, value)?.i();
                (self.node, self.stmt) = (addr(value), addr(s));
                let y = self.land(y, ty)?;
                l.borrow_mut().push(y);
                Ok(Flow::Next)
            }
            Stmt::ForEach { pos, item, list, body } => {
                let (l, ty) = match Self::lookup(env, list) {
                    Some((Val::L(l), t)) => (l, t),
                    _ => return Ok(Flow::Next),
                };
                let mut k = 0;
                while k < l.borrow().len() {
                    let x = l.borrow()[k];
                    let mut scope = HashMap::new();
                    if let Some(p) = pos {
                        scope.insert(p.clone(), (Val::I(k as i128 + 1), Ty::U64.into()));
                    }
                    scope.insert(item.clone(), (Val::I(x), ty));
                    env.push(scope);
                    let f = self.block(env, body);
                    env.pop();
                    match f? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        _ => {}
                    }
                    k += 1;
                }
                Ok(Flow::Next)
            }
            Stmt::Borrow { name, value } => {
                let x = self.expr(env, value)?;
                let ty = match value {
                    Expr::Call(f, _) => self.p.bfuncs.iter().find(|g| &g.name == f).map_or(Ty::I64, |g| g.elem),
                    Expr::Lend(n) => Self::lookup(env, n).map_or(Ty::I64, |v| v.1.base),
                    _ => Ty::I64,
                };
                env.last_mut().unwrap().insert(name.clone(), (x, ty.into()));
                Ok(Flow::Next)
            }
            Stmt::Give { name, from } => {
                let x = Self::lookup(env, from).unwrap_or((Val::I(0), Ty::I64.into()));
                env.last_mut().unwrap().insert(name.clone(), x);
                Ok(Flow::Next)
            }
            Stmt::Loop { body } => loop {
                match self.block(env, body)? {
                    Flow::Break => return Ok(Flow::Next),
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    _ => self.tick()?,
                }
            },
        }
    }
}

fn lanes(v: &Val) -> Vec<i128> {
    match v {
        Val::S(xs) => xs.clone(),
        x => vec![x.clone().i()],
    }
}

fn cmp(op: CmpOp, x: i128, y: i128) -> bool {
    match op {
        CmpOp::Eq => x == y,
        CmpOp::Ne => x != y,
        CmpOp::Lt => x < y,
        CmpOp::Gt => x > y,
        CmpOp::Le => x <= y,
        CmpOp::Ge => x >= y,
    }
}

fn floor_div(a: i128, b: i128) -> i128 {
    let q = a / b;
    if a % b != 0 && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

/// Run `p` on these input values (one per input, in order).
pub fn run(p: &Program, inputs: &[u64]) -> Run {
    let mut ev = Ev { p, seen: HashMap::new(), node: 0, stmt: 0, call_at: 0, steps: 0, depth: 0, ret_ty: vec![] };
    let mut env: Env = vec![HashMap::new()];
    for (i, v) in p.inputs.iter().zip(inputs) {
        env[0].insert(i.name.clone(), (Val::I(*v as i128), Ty::U64.into()));
    }
    let r = ev.block(&mut env, &p.main);
    let (stopped, exhausted) = match r {
        Err(Stop::At((0, 0, "steps"))) => (None, true),
        Err(Stop::At(k)) => (Some(k), false),
        Err(Stop::Leave(_)) | Ok(_) => (None, false),
    };
    Run { seen: ev.seen, stopped, exhausted }
}
