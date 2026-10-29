//! Spacer's second mode: integer clauses (owner, 2026-10-29: "Test out
//! number 2 first").
//!
//! Layer 3's questions are bit-vectors, and Spacer searching in them spends
//! its budget on loops it answers at once over the integers. So in this
//! mode every machine value in the clauses is read as a mathematical
//! integer — signed or unsigned as the encoder made it — with its type's
//! bounds stated as facts, and every operation as the arithmetic it means.
//! What has no integer meaning (a bit operation, a shift) is an unknown.
//! None of that is a proof: Spacer only guesses. The guess is read back into
//! bit-vectors, wide enough that its own arithmetic never wraps, and
//! confirmed there by Z3 and CVC5 exactly as a bit-vector guess is, so a
//! guess that holds only over the integers is rejected there.

use std::collections::HashMap;

/// An s-expression.
#[derive(Clone, Debug)]
pub enum Sx {
    Atom(String),
    List(Vec<Sx>),
}

pub fn parse(text: &str) -> Option<Sx> {
    let b = text.as_bytes();
    let (sx, _) = parse_at(b, 0)?;
    Some(sx)
}

fn parse_at(b: &[u8], mut i: usize) -> Option<(Sx, usize)> {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() {
        return None;
    }
    if b[i] == b'(' {
        let mut items = Vec::new();
        i += 1;
        loop {
            while i < b.len() && b[i].is_ascii_whitespace() {
                i += 1;
            }
            if i >= b.len() {
                return None;
            }
            if b[i] == b')' {
                return Some((Sx::List(items), i + 1));
            }
            let (item, next) = parse_at(b, i)?;
            items.push(item);
            i = next;
        }
    }
    if b[i] == b')' {
        return None;
    }
    let start = i;
    if b[i] == b'|' {
        i += 1;
        while i < b.len() && b[i] != b'|' {
            i += 1;
        }
        i += 1;
    } else {
        while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'(' && b[i] != b')' {
            i += 1;
        }
    }
    Some((Sx::Atom(String::from_utf8_lossy(&b[start..i.min(b.len())]).into_owned()), i))
}

/// 2^w, written out in decimal.
pub fn pow2(w: u32) -> String {
    let mut digits = vec![1u8];
    for _ in 0..w {
        let mut carry = 0;
        for d in digits.iter_mut() {
            let v = *d * 2 + carry;
            *d = v % 10;
            carry = v / 10;
        }
        if carry > 0 {
            digits.push(carry);
        }
    }
    digits.iter().rev().map(|d| (b'0' + d) as char).collect()
}

fn neg(s: &str) -> String {
    format!("(- {})", s)
}

/// A bit-vector term read as an integer: how wide it was, and whether the
/// integer is its signed or its unsigned reading. A literal has neither
/// until the term beside it says which.
#[derive(Clone)]
struct Num {
    e: String,
    w: u32,
    view: View,
}

#[derive(Clone, Copy, PartialEq)]
enum View {
    Signed,
    Unsigned,
    Lit(u128),
}

enum Out {
    Bool(String),
    Num(Num),
}

/// The integer reading of the encoder's lines, one part of a loop's
/// clauses at a time.
pub struct Ints<'a> {
    sorts: &'a HashMap<String, (u32, bool)>,
    /// Unknowns this part needed for what has no integer meaning.
    pub unknowns: Vec<String>,
    next: usize,
}

impl<'a> Ints<'a> {
    pub fn new(sorts: &'a HashMap<String, (u32, bool)>) -> Self {
        Ints { sorts, unknowns: Vec::new(), next: 0 }
    }

    fn unknown(&mut self, w: u32, signed: bool) -> Num {
        let name = format!("unk{}", self.next);
        self.next += 1;
        self.unknowns.push(format!("({} Int)", name));
        Num { e: name, w, view: if signed { View::Signed } else { View::Unsigned } }
    }

    /// An unknown number, for a term this cannot read.
    pub fn fresh_int(&mut self) -> String {
        self.unknown(64, true).e
    }

    /// The integer as read signed, or unsigned.
    fn as_view(&self, n: &Num, signed: bool) -> String {
        match n.view {
            View::Lit(raw) => {
                if signed && n.w <= 128 && n.w > 0 && raw >> (n.w - 1) & 1 == 1 {
                    let v = if n.w == 128 { raw as i128 } else { raw as i128 - (1i128 << n.w) };
                    neg(&v.unsigned_abs().to_string())
                } else {
                    raw.to_string()
                }
            }
            View::Signed if !signed => format!("(ite (< {e} 0) (+ {e} {m}) {e})", e = n.e, m = pow2(n.w)),
            View::Unsigned if signed => format!("(ite (>= {e} {h}) (- {e} {m}) {e})", e = n.e, h = pow2(n.w - 1), m = pow2(n.w)),
            _ => n.e.clone(),
        }
    }

    /// The reading two operands share: the first one that is not a literal's.
    fn common(a: &Num, b: &Num) -> bool {
        match (a.view, b.view) {
            (View::Signed, _) => true,
            (View::Unsigned, _) => false,
            (_, View::Unsigned) => false,
            _ => true,
        }
    }

    /// A bound stated as a fact: what a value of this width holds.
    pub fn range(&self, name: &str) -> Option<String> {
        let (w, signed) = *self.sorts.get(name)?;
        Some(if signed { format!("(and (<= {} {}) (< {} {}))", neg(&pow2(w - 1)), name, name, pow2(w - 1)) } else { format!("(and (<= 0 {}) (< {} {}))", name, name, pow2(w)) })
    }

    /// A Boolean term, or None if it is not one this reads.
    pub fn boolean(&mut self, sx: &Sx) -> Option<String> {
        match self.term(sx)? {
            Out::Bool(b) => Some(b),
            Out::Num(_) => None,
        }
    }

    /// A number, read signed or unsigned.
    pub fn number(&mut self, sx: &Sx, signed: bool) -> Option<String> {
        match self.term(sx)? {
            Out::Num(n) => Some(self.as_view(&n, signed)),
            Out::Bool(_) => None,
        }
    }

    fn num(&mut self, sx: &Sx) -> Option<Num> {
        match self.term(sx)? {
            Out::Num(n) => Some(n),
            Out::Bool(_) => None,
        }
    }

    fn term(&mut self, sx: &Sx) -> Option<Out> {
        match sx {
            Sx::Atom(a) => {
                if a == "true" || a == "false" {
                    return Some(Out::Bool(a.clone()));
                }
                match self.sorts.get(a) {
                    Some(&(w, signed)) => Some(Out::Num(Num { e: a.clone(), w, view: if signed { View::Signed } else { View::Unsigned } })),
                    // A Boolean the encoder defined.
                    None => Some(Out::Bool(a.clone())),
                }
            }
            Sx::List(items) => {
                let head = match items.first()? {
                    Sx::Atom(h) => h.as_str(),
                    // `((_ extract 7 0) x)` and the extensions.
                    Sx::List(inner) => return self.indexed(inner, items.get(1)?),
                };
                let args = &items[1..];
                match head {
                    "_" => {
                        // `(_ bvN w)`
                        let (Sx::Atom(n), Sx::Atom(w)) = (args.first()?, args.get(1)?) else { return None };
                        let raw: u128 = n.strip_prefix("bv")?.parse().ok()?;
                        Some(Out::Num(Num { e: raw.to_string(), w: w.parse().ok()?, view: View::Lit(raw) }))
                    }
                    "and" | "or" | "not" | "=>" | "xor" => {
                        let parts: Option<Vec<String>> = args.iter().map(|a| self.boolean(a)).collect();
                        Some(Out::Bool(format!("({} {})", head, parts?.join(" "))))
                    }
                    "ite" => {
                        let c = self.boolean(args.first()?)?;
                        match (self.term(args.get(1)?)?, self.term(args.get(2)?)?) {
                            (Out::Bool(a), Out::Bool(b)) => Some(Out::Bool(format!("(ite {} {} {})", c, a, b))),
                            (Out::Num(a), Out::Num(b)) => {
                                let s = Self::common(&a, &b);
                                let e = format!("(ite {} {} {})", c, self.as_view(&a, s), self.as_view(&b, s));
                                Some(Out::Num(Num { e, w: a.w.max(b.w), view: if s { View::Signed } else { View::Unsigned } }))
                            }
                            _ => None,
                        }
                    }
                    "=" => match (self.term(args.first()?)?, self.term(args.get(1)?)?) {
                        (Out::Bool(a), Out::Bool(b)) => Some(Out::Bool(format!("(= {} {})", a, b))),
                        (Out::Num(a), Out::Num(b)) => {
                            let s = Self::common(&a, &b);
                            Some(Out::Bool(format!("(= {} {})", self.as_view(&a, s), self.as_view(&b, s))))
                        }
                        _ => None,
                    },
                    "bvslt" | "bvsle" | "bvsgt" | "bvsge" | "bvult" | "bvule" | "bvugt" | "bvuge" => {
                        let (a, b) = (self.num(args.first()?)?, self.num(args.get(1)?)?);
                        let signed = head.starts_with("bvs");
                        let op = match &head[3..] {
                            "lt" => "<",
                            "le" => "<=",
                            "gt" => ">",
                            _ => ">=",
                        };
                        Some(Out::Bool(format!("({} {} {})", op, self.as_view(&a, signed), self.as_view(&b, signed))))
                    }
                    "bvadd" | "bvsub" | "bvmul" => {
                        let (a, b) = (self.num(args.first()?)?, self.num(args.get(1)?)?);
                        let s = Self::common(&a, &b);
                        let op = match head {
                            "bvadd" => "+",
                            "bvsub" => "-",
                            _ => "*",
                        };
                        Some(Out::Num(Num { e: format!("({} {} {})", op, self.as_view(&a, s), self.as_view(&b, s)), w: a.w.max(b.w), view: if s { View::Signed } else { View::Unsigned } }))
                    }
                    "bvneg" => {
                        let a = self.num(args.first()?)?;
                        Some(Out::Num(Num { e: neg(&self.as_view(&a, true)), w: a.w, view: View::Signed }))
                    }
                    "bvsdiv" | "bvsrem" | "bvudiv" | "bvurem" => {
                        let (a, b) = (self.num(args.first()?)?, self.num(args.get(1)?)?);
                        let signed = head.starts_with("bvs");
                        // Only by a literal is it linear; the rounding is
                        // the integers' own, which a guess may get wrong.
                        if !matches!(b.view, View::Lit(_)) {
                            return Some(Out::Num(self.unknown(a.w, signed)));
                        }
                        let op = if head.ends_with("div") { "div" } else { "mod" };
                        Some(Out::Num(Num { e: format!("({} {} {})", op, self.as_view(&a, signed), self.as_view(&b, signed)), w: a.w, view: if signed { View::Signed } else { View::Unsigned } }))
                    }
                    "bvsaddo" | "bvuaddo" | "bvssubo" | "bvusubo" | "bvsmulo" | "bvumulo" => {
                        let (a, b) = (self.num(args.first()?)?, self.num(args.get(1)?)?);
                        let signed = head.starts_with("bvs");
                        let op = match &head[4..6] {
                            "ad" => "+",
                            "su" => "-",
                            _ => "*",
                        };
                        let r = format!("({} {} {})", op, self.as_view(&a, signed), self.as_view(&b, signed));
                        let w = a.w;
                        Some(Out::Bool(if signed { format!("(or (< {r} {}) (>= {r} {}))", neg(&pow2(w - 1)), pow2(w - 1)) } else { format!("(or (< {r} 0) (>= {r} {}))", pow2(w)) }))
                    }
                    "concat" => {
                        let (a, b) = (self.num(args.first()?)?, self.num(args.get(1)?)?);
                        let e = format!("(+ (* {} {}) {})", self.as_view(&a, false), pow2(b.w), self.as_view(&b, false));
                        Some(Out::Num(Num { e, w: a.w + b.w, view: View::Unsigned }))
                    }
                    // A bit operation or a shift: no integer meaning here.
                    "bvand" | "bvor" | "bvxor" | "bvnot" | "bvshl" | "bvlshr" | "bvashr" => {
                        let a = self.num(args.first()?)?;
                        Some(Out::Num(self.unknown(a.w, a.view != View::Unsigned)))
                    }
                    _ => None,
                }
            }
        }
    }

    /// `((_ extract hi lo) x)`, `((_ zero_extend k) x)`, `((_ sign_extend k) x)`.
    fn indexed(&mut self, inner: &[Sx], arg: &Sx) -> Option<Out> {
        let (Sx::Atom(us), Sx::Atom(kind)) = (inner.first()?, inner.get(1)?) else { return None };
        if us != "_" {
            return None;
        }
        let a = self.num(arg)?;
        let n: u32 = match inner.get(2)? {
            Sx::Atom(n) => n.parse().ok()?,
            _ => return None,
        };
        match kind.as_str() {
            "zero_extend" => Some(Out::Num(Num { e: self.as_view(&a, false), w: a.w + n, view: View::Unsigned })),
            "sign_extend" => Some(Out::Num(Num { e: self.as_view(&a, true), w: a.w + n, view: View::Signed })),
            "extract" => {
                let lo: u32 = match inner.get(3)? {
                    Sx::Atom(n) => n.parse().ok()?,
                    _ => return None,
                };
                if lo != 0 {
                    let w = n - lo + 1;
                    return Some(Out::Num(self.unknown(w, false)));
                }
                // Narrowing: the same number, as the integers read it; a
                // value that does not fit is a landing that stopped the
                // program, or a wrap the guess will be confirmed against.
                let w = n + 1;
                match a.view {
                    View::Lit(raw) => {
                        let r = if w >= 128 { raw } else { raw & ((1u128 << w) - 1) };
                        Some(Out::Num(Num { e: r.to_string(), w, view: View::Lit(r) }))
                    }
                    v => Some(Out::Num(Num { e: a.e, w, view: v })),
                }
            }
            _ => None,
        }
    }
}

/// One part of a loop's clauses, over the integers: its variables and its
/// conjuncts, from the encoder's lines.
pub fn part<'a>(defs: &[String], sorts: &'a HashMap<String, (u32, bool)>) -> (Vec<String>, Vec<String>, Ints<'a>) {
    let mut tr = Ints::new(sorts);
    let mut vars = Vec::new();
    let mut conj = Vec::new();
    for d in defs {
        let Some(Sx::List(items)) = parse(d) else { continue };
        let Some(Sx::Atom(kw)) = items.first() else { continue };
        match kw.as_str() {
            "declare-fun" => {
                let Some(Sx::Atom(name)) = items.get(1) else { continue };
                if sorts.contains_key(name) {
                    vars.push(format!("({} Int)", name));
                    conj.extend(tr.range(name));
                } else {
                    vars.push(format!("({} Bool)", name));
                }
            }
            "define-fun" => {
                let (Some(Sx::Atom(name)), Some(body)) = (items.get(1), items.get(4)) else { continue };
                // A computed value has no bound of its own: a narrowing
                // that does not fit is its landing's question, which the
                // encoder states where it is assumed.
                if let Some(&(_, signed)) = sorts.get(name) {
                    vars.push(format!("({} Int)", name));
                    if let Some(v) = tr.number(body, signed) {
                        conj.push(format!("(= {} {})", name, v));
                    }
                } else {
                    vars.push(format!("({} Bool)", name));
                    if let Some(v) = tr.boolean(body) {
                        conj.push(format!("(= {} {})", name, v));
                    }
                }
            }
            "assert" => {
                if let Some(v) = items.get(1).and_then(|b| tr.boolean(b)) {
                    conj.push(v);
                }
            }
            _ => {}
        }
    }
    (vars, conj, tr)
}

/// Spacer's integer invariant, read back into bit-vectors: each parameter
/// widened to `WIDE` bits by its own sign, and the arithmetic done there,
/// where none of it wraps for the numbers an invariant writes.
pub const WIDE: u32 = 192;

pub fn to_bits(body: &Sx, params: &[(String, u32, bool)]) -> Option<String> {
    let mut lets: Vec<HashMap<String, bool>> = Vec::new();
    back(body, params, &mut lets).map(|(s, _)| s)
}

/// The term, and whether it is a number (true) or a Boolean.
fn back(sx: &Sx, params: &[(String, u32, bool)], lets: &mut Vec<HashMap<String, bool>>) -> Option<(String, bool)> {
    let wide = |v: i128| -> String {
        if v < 0 {
            format!("(bvneg (_ bv{} {}))", v.unsigned_abs(), WIDE)
        } else {
            format!("(_ bv{} {})", v, WIDE)
        }
    };
    match sx {
        Sx::Atom(a) => {
            if a == "true" || a == "false" {
                return Some((a.clone(), false));
            }
            if let Ok(v) = a.parse::<i128>() {
                return Some((wide(v), true));
            }
            if let Some((name, bits, signed)) = params.iter().find(|(n, _, _)| n == a) {
                let ext = if *signed { "sign_extend" } else { "zero_extend" };
                return Some((format!("((_ {} {}) {})", ext, WIDE - bits, name), true));
            }
            for scope in lets.iter().rev() {
                if let Some(&num) = scope.get(a) {
                    return Some((a.clone(), num));
                }
            }
            None
        }
        Sx::List(items) => {
            let Sx::Atom(head) = items.first()? else { return None };
            let args = &items[1..];
            let all = |lets: &mut Vec<HashMap<String, bool>>| -> Option<Vec<(String, bool)>> { args.iter().map(|a| back(a, params, lets)).collect() };
            match head.as_str() {
                "let" => {
                    let Sx::List(binds) = args.first()? else { return None };
                    let mut scope = HashMap::new();
                    let mut written = Vec::new();
                    for b in binds {
                        let Sx::List(pair) = b else { return None };
                        let (Sx::Atom(name), value) = (pair.first()?, pair.get(1)?) else { return None };
                        let (v, num) = back(value, params, lets)?;
                        scope.insert(name.clone(), num);
                        written.push(format!("({} {})", name, v));
                    }
                    lets.push(scope);
                    let body = back(args.get(1)?, params, lets);
                    lets.pop();
                    let (body, num) = body?;
                    Some((format!("(let ({}) {})", written.join(" "), body), num))
                }
                "and" | "or" | "not" | "=>" | "xor" => {
                    let parts = all(lets)?;
                    if parts.iter().any(|(_, n)| *n) {
                        return None;
                    }
                    Some((format!("({} {})", head, parts.into_iter().map(|(s, _)| s).collect::<Vec<_>>().join(" ")), false))
                }
                "ite" => {
                    let parts = all(lets)?;
                    if parts.len() != 3 || parts[0].1 {
                        return None;
                    }
                    Some((format!("(ite {} {} {})", parts[0].0, parts[1].0, parts[2].0), parts[1].1))
                }
                "=" | "distinct" => {
                    let parts = all(lets)?;
                    Some((format!("({} {})", head, parts.into_iter().map(|(s, _)| s).collect::<Vec<_>>().join(" ")), false))
                }
                "<" | "<=" | ">" | ">=" => {
                    let parts = all(lets)?;
                    if parts.len() < 2 || parts.iter().any(|(_, n)| !*n) {
                        return None;
                    }
                    let op = match head.as_str() {
                        "<" => "bvslt",
                        "<=" => "bvsle",
                        ">" => "bvsgt",
                        _ => "bvsge",
                    };
                    let chain: Vec<String> = parts.windows(2).map(|w| format!("({} {} {})", op, w[0].0, w[1].0)).collect();
                    Some((if chain.len() == 1 { chain[0].clone() } else { format!("(and {})", chain.join(" ")) }, false))
                }
                "+" | "*" => {
                    let parts = all(lets)?;
                    if parts.iter().any(|(_, n)| !*n) {
                        return None;
                    }
                    let op = if head == "+" { "bvadd" } else { "bvmul" };
                    let mut it = parts.into_iter().map(|(s, _)| s);
                    let first = it.next()?;
                    Some((it.fold(first, |acc, x| format!("({} {} {})", op, acc, x)), true))
                }
                "-" => {
                    let parts = all(lets)?;
                    if parts.iter().any(|(_, n)| !*n) {
                        return None;
                    }
                    let mut it = parts.into_iter().map(|(s, _)| s);
                    let first = it.next()?;
                    let rest: Vec<String> = it.collect();
                    if rest.is_empty() {
                        return Some((format!("(bvneg {})", first), true));
                    }
                    Some((rest.into_iter().fold(first, |acc, x| format!("(bvsub {} {})", acc, x)), true))
                }
                "div" | "mod" => {
                    // By a positive literal only: then the integers' `mod`
                    // is `bvsmod`'s, and `div` the exact quotient after it.
                    let (Some(Sx::Atom(c)), Some(a)) = (args.get(1), args.first()) else { return None };
                    let c: i128 = c.parse().ok().filter(|c: &i128| *c > 0)?;
                    let (a, num) = back(a, params, lets)?;
                    if !num {
                        return None;
                    }
                    let m = format!("(bvsmod {} {})", a, wide(c));
                    Some((if head == "mod" { m } else { format!("(bvsdiv (bvsub {} {}) {})", a, m, wide(c)) }, true))
                }
                _ => None,
            }
        }
    }
}
