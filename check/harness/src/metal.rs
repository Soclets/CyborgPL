//! `inline.MSL` against the oracle (kayo 793861b5, 7aed734d): the oracle
//! cannot run Metal, so each case is two programs. (A) runs kernels on
//! Apple's GPU, one thread per item; (B) works the same steps out in plain
//! loops. B runs on the oracle and on kayo, A on kayo; all three must print
//! the same. What is compared is kayo's Metal translation and Apple's GPU
//! against the oracle's arithmetic.
//!
//! A case is 1 to 3 kernels in a row, each its own `inline.MSL` block: the
//! first reads two lists, or one list of a `layout.C` struct `Pr { a, b }`
//! (kayo writes its Metal declaration), and each later one reads the list the
//! one before it wrote, beside the first `y`. The last writes a list, or,
//! with the struct, its result into each struct's `a` through `lend.mut`.
//! One case in eight has lists of 4096 to 6000 items built by appends, which
//! kayo hands to the GPU without a copy (7aed734d); the results must not
//! change.
//!
//! Each step is its own declared value (`uint t3 = t1 + t2;` against `let
//! $t3: u32 = $t1 plus.wrap $t2;`), so the two read alike; a stage's result
//! always takes in the item's own x and y. The values are of one type a case:
//! - `u32`: wrapping `+ - *`, `/` and `%` by a literal not zero, `& | ^`,
//!   `>>` under 32; the block plain, `.precise`, `.wrap` or both, none of which
//!   may change an unsigned answer;
//! - `i32` in `inline.MSL.wrap` (`-fwrapv`, so signed overflow wraps, as
//!   CyborgPL's `.wrap` words do): the same, `/` and `%` truncating and `>>`
//!   keeping the sign on both sides;
//! - `f32` in `inline.MSL.precise` (no fast math, no fused multiply-add):
//!   `+ - *` on multiples of 1/8 up to 100, so every value is exact, none near
//!   a denormal (safe mode may still flush those), and a product multiplies
//!   only by an input, so nothing outgrows `f32`.
//!
//! Left out: `<<`, as CyborgPL checks what it shifts out, and signed overflow
//! in a plain block, undefined in Metal.

use crate::engines::{self, Tools};
use check_gen::rng::Rng;
use std::path::Path;

#[derive(Clone, Copy, PartialEq)]
enum Ty {
    U32,
    I32,
    F32,
}

impl Ty {
    fn metal(self) -> &'static str {
        match self {
            Ty::U32 => "uint",
            Ty::I32 => "int",
            Ty::F32 => "float",
        }
    }
    fn cy(self) -> &'static str {
        match self {
            Ty::U32 => "u32",
            Ty::I32 => "i32",
            Ty::F32 => "f32",
        }
    }
}

/// A value of the case's type: its CyborgPL literal and its Metal one.
#[derive(Clone)]
struct Val {
    cy: String,
    metal: String,
}

#[derive(Clone)]
enum Step {
    /// `+ - *`: the Metal symbol, the CyborgPL word
    Arith(&'static str, &'static str, usize, usize),
    Bits(&'static str, &'static str, usize, usize),
    ByLit(&'static str, &'static str, usize, u32),
    Shr(usize, u32),
    Lit(Val),
}

#[derive(Clone, Copy, PartialEq)]
enum Form {
    /// two lists in, a list out
    Lists,
    /// a list of `Pr` in, a list out
    StructIn,
    /// a list of `Pr` in, the last stage's result written into each `.a`
    StructOut,
}

pub struct Case {
    ty: Ty,
    /// the block's word: `inline.MSL`, `.precise`, `.wrap` or `.precise.wrap`
    block: &'static str,
    xs: Vec<Val>,
    ys: Vec<Val>,
    k: Val,
    /// lists built by appends to this many items, in place of `xs` / `ys`
    big: Option<(usize, u32, u32)>,
    form: Form,
    stages: Vec<Vec<Step>>,
}

fn word(r: &mut Rng, ty: Ty) -> Val {
    match ty {
        Ty::U32 => {
            let v = match r.below(4) {
                0 => r.range(0, 16) as u32,
                1 => u32::MAX - r.range(0, 16) as u32,
                _ => r.range(0, u32::MAX as i128) as u32,
            };
            Val { cy: v.to_string(), metal: format!("{v}u") }
        }
        Ty::I32 => {
            // near both ends as well as anywhere; never i32's least value,
            // which Metal cannot write as one literal
            let v: i64 = match r.below(4) {
                0 => r.range(-16, 16) as i64,
                1 => i32::MAX as i64 - r.range(0, 16) as i64,
                2 => i32::MIN as i64 + 1 + r.range(0, 16) as i64,
                _ => r.range(i32::MIN as i128 + 1, i32::MAX as i128) as i64,
            };
            Val { cy: if v < 0 { format!("({v})") } else { v.to_string() }, metal: v.to_string() }
        }
        Ty::F32 => {
            // a multiple of 1/8 up to 100, not zero
            let mut m = r.range(-800, 800);
            if m == 0 {
                m = 1;
            }
            let v = m as f64 / 8.0;
            let s = format!("{v:?}");
            Val { cy: if v < 0.0 { format!("({s})") } else { s.clone() }, metal: format!("{s}f") }
        }
    }
}

/// Values 0, 1, 2 are x, y and k; each step makes the next one.
fn steps(r: &mut Rng, ty: Ty) -> Vec<Step> {
    let arith = |r: &mut Rng, a: usize, b: usize| -> Step {
        let (m, w) = *r.pick(&if ty == Ty::F32 { [("+", "+"), ("-", "-"), ("*", "x")] } else { [("+", "plus.wrap"), ("-", "minus.wrap"), ("*", "times.wrap")] });
        // a float product multiplies only by an input, so it stays in range
        let b = if ty == Ty::F32 && m == "*" { b % 3 } else { b };
        Step::Arith(m, w, a, b)
    };
    let mut steps = vec![];
    for _ in 0..r.range(1, 10) {
        let have = 3 + steps.len();
        // half the time a step works on the one before it, so chains go deep
        let a = if r.chance(50) { have - 1 } else { r.below(have as u64) as usize };
        let b = r.below(have as u64) as usize;
        let s = match (ty, r.below(6)) {
            (_, 0 | 1) | (Ty::F32, 2..=4) => arith(r, a, b),
            (_, 2) => {
                let (m, w) = *r.pick(&[("&", "and.bit"), ("|", "or.bit"), ("^", "xor")]);
                Step::Bits(m, w, a, b)
            }
            (_, 3) => {
                let d = r.range(1, 1000) as u32;
                let (m, w) = *r.pick(&[("/", "div.trunc"), ("%", "mod.trunc")]);
                Step::ByLit(m, w, a, d)
            }
            (_, 4) => Step::Shr(a, r.range(0, 31) as u32),
            _ => Step::Lit(word(r, ty)),
        };
        steps.push(s);
    }
    // the result takes in the item's own x, then y, so each thread's answer
    // is its own and it never ends on a literal
    for own in [0, 1] {
        let have = 3 + steps.len();
        let s = if ty == Ty::F32 || r.chance(75) { arith(r, have - 1, own) } else { Step::Bits("^", "xor", have - 1, own) };
        steps.push(s);
    }
    steps
}

fn name(i: usize) -> String {
    match i {
        0 => "x".into(),
        1 => "y".into(),
        2 => "k".into(),
        _ => format!("t{i}"),
    }
}

impl Case {
    pub fn random(seed: u64) -> Case {
        let mut r = Rng::new(seed);
        let ty = *r.pick(&[Ty::U32, Ty::U32, Ty::I32, Ty::F32]);
        let block = match ty {
            Ty::U32 => *r.pick(&["inline.MSL", "inline.MSL.precise", "inline.MSL.wrap", "inline.MSL.precise.wrap"]),
            Ty::I32 => *r.pick(&["inline.MSL.wrap", "inline.MSL.precise.wrap"]),
            Ty::F32 => *r.pick(&["inline.MSL.precise", "inline.MSL.precise.wrap"]),
        };
        let n = r.range(1, 64) as usize;
        let xs = (0..n).map(|_| word(&mut r, ty)).collect();
        let ys = (0..n).map(|_| word(&mut r, ty)).collect();
        let k = word(&mut r, ty);
        let form = *r.pick(&[Form::Lists, Form::Lists, Form::StructIn, Form::StructOut]);
        let big = (form == Form::Lists && ty != Ty::F32 && r.chance(12)).then(|| (r.range(4096, 6000) as usize, r.range(1, 99999) as u32, r.range(0, 99999) as u32));
        let stages = (0..r.range(1, 3)).map(|_| steps(&mut r, ty)).collect();
        Case { ty, block, xs, ys, k, big, form, stages }
    }

    fn list(&self, v: &[Val]) -> String {
        format!("[{},]", v.iter().map(|x| x.cy.clone()).collect::<Vec<_>>().join(", "))
    }

    fn head(&self) -> String {
        let t = self.ty.cy();
        let mut s = String::new();
        if self.form != Form::Lists {
            s.push_str(&format!("layout.C struct Pr {{ a: {t}, b: {t} }}\n\n"));
        }
        s.push_str("MAIN {\n");
        match (self.form, self.big) {
            (Form::Lists, Some((n, a, b))) => {
                // built by appends: lists of 16 KB and more go to the GPU uncopied
                s.push_str(&format!("    let mut $xs: std::list[{t}] = std::list.empty;\n    let mut $ys: std::list[{t}] = std::list.empty;\n"));
                // the counter of the lists' own type, so no conversion is needed
                s.push_str(&format!("    for $i: {t} [from: 1, to: {n}, by: 1] {{\n        let $w: {t} = $i;\n        let $v: {t} = $w times.wrap {a};\n        std::list.append(lend.mut $xs, value: $v plus.wrap {b});\n        std::list.append(lend.mut $ys, value: $v xor $w);\n    }}\n"));
            }
            (Form::Lists, None) => s.push_str(&format!("    let $xs: std::list[{t}] = {};\n    let $ys: std::list[{t}] = {};\n", self.list(&self.xs), self.list(&self.ys))),
            _ => {
                let items: Vec<String> = self.xs.iter().zip(&self.ys).map(|(a, b)| format!("Pr[a = {}, b = {}]", a.cy, b.cy)).collect();
                // `mut` only where the last kernel writes into them
                let m = if self.form == Form::StructOut { "mut " } else { "" };
                s.push_str(&format!("    let {m}$prs: std::list[Pr] = [{},];\n", items.join(", ")));
            }
        }
        s.push_str(&format!("    let $k: {t} = {};\n", self.k.cy));
        s
    }

    fn count(&self) -> &'static str {
        if self.form == Form::Lists { "std::list.length(lend $xs)" } else { "std::list.length(lend $prs)" }
    }

    fn tail(&self) -> String {
        if self.form == Form::StructOut {
            "    for $p in lend $prs { std::print.useful($p.a, \" \", $p.b, newln); }\n}\n".to_string()
        } else {
            format!("    std::print.useful($o{}, newln);\n}}\n", self.stages.len())
        }
    }

    /// Where stage `j`'s x and y come from, in Metal or in CyborgPL.
    fn inputs(&self, j: usize, metal: bool) -> (String, String) {
        let (x0, y0, prev, a, b) = if metal {
            ("xs[id]".to_string(), "ys[id]".to_string(), "prev[id]".to_string(), "prs[id].a".to_string(), "prs[id].b".to_string())
        } else {
            // a struct's fields are read from a copy of the item (`$p`, declared first)
            ("$xs[pos: $i]".to_string(), "$ys[pos: $i]".to_string(), format!("$o{j}[pos: $i]"), "$p.a".to_string(), "$p.b".to_string())
        };
        match (j == 0, self.form) {
            (true, Form::Lists) => (x0, y0),
            (true, _) => (a, b),
            (false, Form::Lists) => (prev, y0),
            (false, _) => (prev, b),
        }
    }

    /// (A): the kernels on the GPU.
    pub fn gpu(&self) -> String {
        let (mt, ct) = (self.ty.metal(), self.ty.cy());
        let mut s = self.head();
        for (j, st) in self.stages.iter().enumerate() {
            let writes_structs = j + 1 == self.stages.len() && self.form == Form::StructOut;
            let (xin, yin) = self.inputs(j, true);
            let mut body = format!("            {mt} x = {xin};\n            {mt} y = {yin};\n");
            for (i, step) in st.iter().enumerate() {
                let line = match step {
                    Step::Arith(m, _, a, b) | Step::Bits(m, _, a, b) => format!("{} {m} {}", name(*a), name(*b)),
                    Step::ByLit(m, _, a, d) => format!("{} {m} {d}", name(*a)),
                    Step::Shr(a, c) => format!("{} >> {c}", name(*a)),
                    Step::Lit(v) => v.metal.clone(),
                };
                body.push_str(&format!("            {mt} {} = {line};\n", name(3 + i)));
            }
            let result = name(2 + st.len());
            // the buffers, in order: what it reads, what it writes, then k;
            // read-only ones `device const`, as a `lend` must be (7aed734d)
            let mut bufs: Vec<(String, String)> = vec![];
            let structs = || -> (String, String) {
                if writes_structs { ("device Pr *prs".into(), "lend.mut $prs".into()) } else { ("device const Pr *prs".into(), "lend $prs".into()) }
            };
            match (j == 0, self.form) {
                (true, Form::Lists) => {
                    bufs.push((format!("device const {mt} *xs"), "lend $xs".into()));
                    bufs.push((format!("device const {mt} *ys"), "lend $ys".into()));
                }
                (true, _) => bufs.push(structs()),
                (false, Form::Lists) => {
                    bufs.push((format!("device const {mt} *prev"), format!("lend $o{j}")));
                    bufs.push((format!("device const {mt} *ys"), "lend $ys".into()));
                }
                (false, _) => {
                    bufs.push((format!("device const {mt} *prev"), format!("lend $o{j}")));
                    bufs.push(structs());
                }
            }
            if writes_structs {
                body.push_str(&format!("            prs[id].a = {result};\n"));
            } else {
                bufs.push((format!("device {mt} *out"), format!("lend.mut $o{}", j + 1)));
                body.push_str(&format!("            out[id] = {result};\n"));
                let zero = if self.ty == Ty::F32 { "0.0" } else { "0" };
                s.push_str(&format!("    let mut $o{}: std::list[{ct}] = std::list.fill(value: {zero}, count: {});\n", j + 1, self.count()));
            }
            bufs.push((format!("constant {mt} &k"), "copy $k".into()));
            let params: Vec<String> = bufs.iter().enumerate().map(|(n, (p, _))| format!("{p} [[buffer({n})]]")).collect();
            let binds: Vec<String> = bufs.iter().enumerate().map(|(n, (_, w))| format!("{w} -> buffer({n})")).collect();
            s.push_str(&format!(
                "    no_check {} threads {} {{\n        kernel void stage{}({}, uint id [[thread_position_in_grid]]) {{\n{body}        }}\n    }} with [{}];\n",
                self.block,
                self.count(),
                j + 1,
                params.join(", "),
                binds.join(", ")
            ));
        }
        s.push_str(&self.tail());
        s
    }

    /// (B): the same steps in loops.
    pub fn cpu(&self) -> String {
        let ct = self.ty.cy();
        let mut s = self.head();
        for (j, st) in self.stages.iter().enumerate() {
            let writes_structs = j + 1 == self.stages.len() && self.form == Form::StructOut;
            let (xin, yin) = self.inputs(j, false);
            let mut body = String::new();
            if self.form != Form::Lists {
                body.push_str("        let $p := copy $prs[pos: $i];\n");
            }
            body.push_str(&format!("        let $x: {ct} = {xin};\n        let $y: {ct} = {yin};\n"));
            for (i, step) in st.iter().enumerate() {
                let v = |n: usize| format!("${}", name(n));
                let line = match step {
                    Step::Arith(_, w, a, b) | Step::Bits(_, w, a, b) => format!("{} {w} {}", v(*a), v(*b)),
                    Step::ByLit(_, w, a, d) => format!("{} {w} {d}", v(*a)),
                    Step::Shr(a, c) => format!("{} shr {c}", v(*a)),
                    Step::Lit(n) => n.cy.clone(),
                };
                body.push_str(&format!("        let ${}: {ct} = {line};\n", name(3 + i)));
            }
            let result = format!("${}", name(2 + st.len()));
            if writes_structs {
                // the structs rebuilt with the result in `a`, as the kernel leaves them
                s.push_str("    let mut $done: std::list[Pr] = std::list.empty;\n");
                body.push_str(&format!("        std::list.append(lend.mut $done, value: Pr[a = {result}, b = $p.b]);\n"));
            } else {
                s.push_str(&format!("    let mut $o{}: std::list[{ct}] = std::list.empty;\n", j + 1));
                body.push_str(&format!("        std::list.append(lend.mut $o{}, value: {result});\n", j + 1));
            }
            s.push_str(&format!("    for $i: u64 [from: 1, to: {}, by: 1] {{\n{body}    }}\n", self.count()));
            if writes_structs {
                s.push_str("    change $prs = give $done;\n");
            }
        }
        s.push_str(&self.tail());
        s
    }
}

pub enum Outcome {
    Same,
    Differ(String),
    /// a program did not build or run: which, and why
    Failed(String),
}

fn kayo(t: &Tools, file: &Path, dir: &Path) -> Result<Vec<u8>, String> {
    match engines::cca_build(t, file, dir, engines::Mode::Normal) {
        engines::Build::Built(bin) => match engines::cca_run(&bin, dir, &[]) {
            engines::Exec::Ran { out } => Ok(out),
            e => Err(crate::compare::exec_text(&e)),
        },
        engines::Build::Refused { msg, .. } | engines::Build::NotYet { msg } => Err(msg),
        engines::Build::Timeout => Err("build timed out".into()),
    }
}

pub fn check(t: &Tools, c: &Case, dir: &Path) -> Outcome {
    let (da, db) = (dir.join("gpu"), dir.join("cpu"));
    for d in [&da, &db] {
        let _ = std::fs::create_dir_all(d);
    }
    let (fa, fb) = (da.join("prog.cy"), db.join("prog.cy"));
    if std::fs::write(&fa, c.gpu()).is_err() || std::fs::write(&fb, c.cpu()).is_err() {
        return Outcome::Failed("cannot write the files".into());
    }
    let o = match engines::oracle(t, &fb, &db, &[]) {
        engines::Oracle::Ran { out } => out,
        o => return Outcome::Failed(format!("oracle, the loops: {}", crate::compare::oracle_text(&o))),
    };
    let kb = match kayo(t, &fb, &db) {
        Ok(out) => out,
        Err(e) => return Outcome::Failed(format!("kayo, the loops: {e}")),
    };
    let ka = match kayo(t, &fa, &da) {
        Ok(out) => out,
        Err(e) => return Outcome::Failed(format!("kayo, the kernels: {e}")),
    };
    let s = |v: &[u8]| {
        let t = String::from_utf8_lossy(v).trim_end().to_string();
        if t.len() > 300 { format!("{}…", &t[..300]) } else { t }
    };
    if o != kb {
        return Outcome::Differ(format!("the loops: oracle {}, kayo {}", s(&o), s(&kb)));
    }
    if ka != o {
        return Outcome::Differ(format!("the kernels on the GPU {}, the loops {}", s(&ka), s(&o)));
    }
    Outcome::Same
}
