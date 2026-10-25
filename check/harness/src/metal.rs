//! `inline.MSL` against the oracle (kayo 793861b5): the oracle cannot run
//! Metal, so each case is two programs. (A) runs kernels on Apple's GPU, one
//! thread per item; (B) works the same steps out in plain loops. B runs on
//! the oracle and on kayo, A on kayo; all three must print the same. What is
//! compared is kayo's Metal translation and Apple's GPU against the oracle's
//! arithmetic.
//!
//! A case is 1 to 3 kernels in a row, each its own `inline.MSL` block: the
//! first reads two `u32` lists, or one list of a `layout.C` struct `Pr { a:
//! u32, b: u32 }` (kayo writes its Metal declaration), and each later one reads
//! the list the one before it wrote, beside the first `y`. The last writes a
//! `u32` list, or, with the struct, its result into each struct's `a` through
//! `lend.mut`.
//!
//! The steps are `u32` only, where the meaning is exact on both sides:
//! wrapping `+ - *`, `/` and `%` by a literal that is not zero, `& | ^`, and
//! `>>` by under 32. Each step is its own declared value (`uint t3 = t1 +
//! t2;` against `let $t3: u32 = $t1 plus.wrap $t2;`), so the two read alike;
//! a stage's result always takes in the item's own x and y. Left out until
//! ruled (compiler claude, 2026-10-25): floats, as kayo compiles Metal with
//! Apple's fast math, and signed overflow, undefined in Metal (`add nsw`);
//! and `<<`, as CyborgPL checks what it shifts out.

use crate::engines::{self, Tools};
use check_gen::rng::Rng;
use std::path::Path;

#[derive(Clone, Copy)]
enum Step {
    Wrap(&'static str, &'static str, usize, usize),
    Bits(&'static str, &'static str, usize, usize),
    ByLit(&'static str, &'static str, usize, u32),
    Shr(usize, u32),
    Lit(u32),
}

#[derive(Clone, Copy, PartialEq)]
enum Form {
    /// two `u32` lists in, a `u32` list out
    Lists,
    /// a list of `Pr` in, a `u32` list out
    StructIn,
    /// a list of `Pr` in, the last stage's result written into each `.a`
    StructOut,
}

pub struct Case {
    xs: Vec<u32>,
    ys: Vec<u32>,
    k: u32,
    form: Form,
    stages: Vec<Vec<Step>>,
}

fn word(r: &mut Rng) -> u32 {
    match r.below(4) {
        0 => r.range(0, 16) as u32,
        1 => u32::MAX - r.range(0, 16) as u32,
        _ => r.range(0, u32::MAX as i128) as u32,
    }
}

/// Values 0, 1, 2 are x, y and k; each step makes the next one.
fn steps(r: &mut Rng) -> Vec<Step> {
    let mut steps = vec![];
    for _ in 0..r.range(1, 10) {
        let have = 3 + steps.len();
        // half the time a step works on the one before it, so chains go deep
        let a = if r.chance(50) { have - 1 } else { r.below(have as u64) as usize };
        let b = r.below(have as u64) as usize;
        let s = match r.below(6) {
            0 | 1 => *r.pick(&[Step::Wrap("+", "plus.wrap", a, b), Step::Wrap("-", "minus.wrap", a, b), Step::Wrap("*", "times.wrap", a, b)]),
            2 => *r.pick(&[Step::Bits("&", "and.bit", a, b), Step::Bits("|", "or.bit", a, b), Step::Bits("^", "xor", a, b)]),
            3 => {
                let d = r.range(1, 1000) as u32;
                *r.pick(&[Step::ByLit("/", "div.trunc", a, d), Step::ByLit("%", "mod.trunc", a, d)])
            }
            4 => Step::Shr(a, r.range(0, 31) as u32),
            _ => Step::Lit(word(r)),
        };
        steps.push(s);
    }
    // the result takes in the item's own x, then y, so each thread's answer
    // is its own and it never ends on a literal
    for own in [0, 1] {
        let have = 3 + steps.len();
        steps.push(*r.pick(&[Step::Wrap("+", "plus.wrap", have - 1, own), Step::Bits("^", "xor", have - 1, own), Step::Wrap("*", "times.wrap", have - 1, own), Step::Wrap("-", "minus.wrap", have - 1, own)]));
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

fn list(v: &[u32]) -> String {
    format!("[{},]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
}

impl Case {
    pub fn random(seed: u64) -> Case {
        let mut r = Rng::new(seed);
        let n = r.range(1, 64) as usize;
        let xs = (0..n).map(|_| word(&mut r)).collect();
        let ys = (0..n).map(|_| word(&mut r)).collect();
        let k = word(&mut r);
        let form = *r.pick(&[Form::Lists, Form::Lists, Form::StructIn, Form::StructOut]);
        let stages = (0..r.range(1, 3)).map(|_| steps(&mut r)).collect();
        Case { xs, ys, k, form, stages }
    }

    fn head(&self) -> String {
        let mut s = String::new();
        if self.form != Form::Lists {
            s.push_str("layout.C struct Pr { a: u32, b: u32 }\n\n");
        }
        s.push_str("MAIN {\n");
        if self.form == Form::Lists {
            s.push_str(&format!("    let $xs: std::list[u32] = {};\n    let $ys: std::list[u32] = {};\n", list(&self.xs), list(&self.ys)));
        } else {
            let items: Vec<String> = self.xs.iter().zip(&self.ys).map(|(a, b)| format!("Pr[a = {a}, b = {b}]")).collect();
            // `mut` only where the last kernel writes into them
            let m = if self.form == Form::StructOut { "mut " } else { "" };
            s.push_str(&format!("    let {m}$prs: std::list[Pr] = [{},];\n", items.join(", ")));
        }
        s.push_str(&format!("    let $k: u32 = {};\n", self.k));
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
        let (x0, y0, prev, ys, a, b) = if metal {
            ("xs[id]".to_string(), "ys[id]".to_string(), "prev[id]".to_string(), "ys[id]".to_string(), "prs[id].a".to_string(), "prs[id].b".to_string())
        } else {
            // a struct's fields are read from a copy of the item (`$p`, declared first)
            ("$xs[pos: $i]".to_string(), "$ys[pos: $i]".to_string(), format!("$o{j}[pos: $i]"), "$ys[pos: $i]".to_string(), "$p.a".to_string(), "$p.b".to_string())
        };
        match (j == 0, self.form) {
            (true, Form::Lists) => (x0, y0),
            (true, _) => (a, b),
            (false, Form::Lists) => (prev, ys),
            (false, _) => (prev, b),
        }
    }

    /// (A): the kernels on the GPU.
    pub fn gpu(&self) -> String {
        let mut s = self.head();
        for (j, st) in self.stages.iter().enumerate() {
            let writes_structs = j + 1 == self.stages.len() && self.form == Form::StructOut;
            let (xin, yin) = self.inputs(j, true);
            let mut body = format!("            uint x = {xin};\n            uint y = {yin};\n");
            for (i, step) in st.iter().enumerate() {
                let line = match *step {
                    Step::Wrap(m, _, a, b) | Step::Bits(m, _, a, b) => format!("{} {m} {}", name(a), name(b)),
                    Step::ByLit(m, _, a, d) => format!("{} {m} {d}u", name(a)),
                    Step::Shr(a, c) => format!("{} >> {c}u", name(a)),
                    Step::Lit(v) => format!("{v}u"),
                };
                body.push_str(&format!("            uint {} = {line};\n", name(3 + i)));
            }
            let result = name(2 + st.len());
            // the buffers, in order: what it reads, what it writes, then k
            let mut bufs: Vec<(String, String)> = vec![];
            let structs = || -> (String, String) {
                if writes_structs { ("device Pr *prs".into(), "lend.mut $prs".into()) } else { ("device const Pr *prs".into(), "lend $prs".into()) }
            };
            match (j == 0, self.form) {
                (true, Form::Lists) => {
                    bufs.push(("device const uint *xs".into(), "lend $xs".into()));
                    bufs.push(("device const uint *ys".into(), "lend $ys".into()));
                }
                (true, _) => bufs.push(structs()),
                (false, Form::Lists) => {
                    bufs.push(("device const uint *prev".into(), format!("lend $o{j}")));
                    bufs.push(("device const uint *ys".into(), "lend $ys".into()));
                }
                (false, _) => {
                    bufs.push(("device const uint *prev".into(), format!("lend $o{j}")));
                    bufs.push(structs());
                }
            }
            if writes_structs {
                body.push_str(&format!("            prs[id].a = {result};\n"));
            } else {
                bufs.push(("device uint *out".into(), format!("lend.mut $o{}", j + 1)));
                body.push_str(&format!("            out[id] = {result};\n"));
                s.push_str(&format!("    let mut $o{}: std::list[u32] = std::list.fill(value: 0, count: {});\n", j + 1, self.count()));
            }
            bufs.push(("constant uint &k".into(), "copy $k".into()));
            let params: Vec<String> = bufs.iter().enumerate().map(|(n, (p, _))| format!("{p} [[buffer({n})]]")).collect();
            let binds: Vec<String> = bufs.iter().enumerate().map(|(n, (_, w))| format!("{w} -> buffer({n})")).collect();
            s.push_str(&format!(
                "    no_check inline.MSL threads {} {{\n        kernel void stage{}({}, uint id [[thread_position_in_grid]]) {{\n{body}        }}\n    }} with [{}];\n",
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
        let mut s = self.head();
        for (j, st) in self.stages.iter().enumerate() {
            let writes_structs = j + 1 == self.stages.len() && self.form == Form::StructOut;
            let (xin, yin) = self.inputs(j, false);
            let mut body = String::new();
            if self.form != Form::Lists {
                body.push_str("        let $p := copy $prs[pos: $i];\n");
            }
            body.push_str(&format!("        let $x: u32 = {xin};\n        let $y: u32 = {yin};\n"));
            for (i, step) in st.iter().enumerate() {
                let v = |n: usize| format!("${}", name(n));
                let line = match *step {
                    Step::Wrap(_, w, a, b) | Step::Bits(_, w, a, b) => format!("{} {w} {}", v(a), v(b)),
                    Step::ByLit(_, w, a, d) => format!("{} {w} {d}", v(a)),
                    Step::Shr(a, c) => format!("{} shr {c}", v(a)),
                    Step::Lit(n) => n.to_string(),
                };
                body.push_str(&format!("        let ${}: u32 = {line};\n", name(3 + i)));
            }
            let result = format!("${}", name(2 + st.len()));
            if writes_structs {
                // the structs rebuilt with the result in `a`, as the kernel leaves them
                s.push_str("    let mut $done: std::list[Pr] = std::list.empty;\n");
                body.push_str(&format!("        std::list.append(lend.mut $done, value: Pr[a = {result}, b = $p.b]);\n"));
            } else {
                s.push_str(&format!("    let mut $o{}: std::list[u32] = std::list.empty;\n", j + 1));
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
    let s = |v: &[u8]| String::from_utf8_lossy(v).trim_end().to_string();
    if o != kb {
        return Outcome::Differ(format!("the loops: oracle {}, kayo {}", s(&o), s(&kb)));
    }
    if ka != o {
        return Outcome::Differ(format!("the kernels on the GPU {}, the loops {}", s(&ka), s(&o)));
    }
    Outcome::Same
}
