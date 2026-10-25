//! `inline.MSL` against the oracle (kayo 793861b5): the oracle cannot run
//! Metal, so each case is two programs. (A) runs a kernel on Apple's GPU,
//! one thread per item, writing its result into a list; (B) works the same
//! steps out in a plain loop. B runs on the oracle and on kayo, A on kayo;
//! all three must print the same list. What is compared is kayo's Metal
//! translation and Apple's GPU against the oracle's arithmetic.
//!
//! The steps are `u32` only, where the meaning is exact on both sides:
//! wrapping `+ - *`, `/` and `%` by a literal that is not zero, `& | ^`, and
//! `>>` by under 32. Each step is its own declared value (`uint t3 = t1 +
//! t2;` against `let $t3: u32 = $t1 plus.wrap $t2;`), so the two read alike.
//! Left out: signed overflow (undefined in Metal), `<<` (CyborgPL checks what
//! it shifts out), floats (Metal may compile them fast, not exactly).

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

pub struct Case {
    xs: Vec<u32>,
    ys: Vec<u32>,
    k: u32,
    steps: Vec<Step>,
}

impl Case {
    pub fn random(seed: u64) -> Case {
        let mut r = Rng::new(seed);
        let n = r.range(1, 64) as usize;
        let word = |r: &mut Rng| -> u32 {
            match r.below(4) {
                0 => r.range(0, 16) as u32,
                1 => u32::MAX - r.range(0, 16) as u32,
                _ => r.range(0, u32::MAX as i128) as u32,
            }
        };
        let xs = (0..n).map(|_| word(&mut r)).collect();
        let ys = (0..n).map(|_| word(&mut r)).collect();
        let k = word(&mut r);
        // values 0, 1, 2 are x, y and k; each step makes the next one
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
                _ => Step::Lit(word(&mut r)),
            };
            steps.push(s);
        }
        // the result takes in the item's own x, then y, so each thread's answer
        // is its own and it never ends on a literal
        for own in [0, 1] {
            let have = 3 + steps.len();
            steps.push(*r.pick(&[Step::Wrap("+", "plus.wrap", have - 1, own), Step::Bits("^", "xor", have - 1, own), Step::Wrap("*", "times.wrap", have - 1, own), Step::Wrap("-", "minus.wrap", have - 1, own)]));
        }
        Case { xs, ys, k, steps }
    }

    fn name(i: usize) -> String {
        match i {
            0 => "x".into(),
            1 => "y".into(),
            2 => "k".into(),
            _ => format!("t{}", i),
        }
    }

    fn list(v: &[u32]) -> String {
        format!("[{},]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
    }

    fn lets(&self) -> String {
        format!(
            "    let $xs: std::list[u32] = {};\n    let $ys: std::list[u32] = {};\n    let $k: u32 = {};\n",
            Self::list(&self.xs),
            Self::list(&self.ys),
            self.k
        )
    }

    fn last(&self) -> usize {
        2 + self.steps.len()
    }

    /// (A): the kernel on the GPU.
    pub fn gpu(&self) -> String {
        let mut body = String::from("            uint x = xs[id];\n            uint y = ys[id];\n");
        for (j, s) in self.steps.iter().enumerate() {
            let t = Self::name(3 + j);
            let line = match *s {
                Step::Wrap(m, _, a, b) | Step::Bits(m, _, a, b) => format!("{} {m} {}", Self::name(a), Self::name(b)),
                Step::ByLit(m, _, a, d) => format!("{} {m} {d}u", Self::name(a)),
                Step::Shr(a, c) => format!("{} >> {c}u", Self::name(a)),
                Step::Lit(v) => format!("{v}u"),
            };
            body.push_str(&format!("            uint {t} = {line};\n"));
        }
        body.push_str(&format!("            out[id] = {};\n", Self::name(self.last())));
        format!(
            "MAIN {{\n{}    let mut $out: std::list[u32] = std::list.fill(value: 0, count: std::list.length(lend $xs));\n    no_check inline.MSL threads std::list.length(lend $xs) {{\n        kernel void steps(device const uint *xs [[buffer(0)]], device const uint *ys [[buffer(1)]], device uint *out [[buffer(2)]], constant uint &k [[buffer(3)]], uint id [[thread_position_in_grid]]) {{\n{body}        }}\n    }} with [lend $xs -> buffer(0), lend $ys -> buffer(1), lend.mut $out -> buffer(2), copy $k -> buffer(3)];\n    std::print.useful($out, newln);\n}}\n",
            self.lets()
        )
    }

    /// (B): the same steps in a loop.
    pub fn cpu(&self) -> String {
        let mut body = String::from("        let $x: u32 = $xs[pos: $i];\n        let $y: u32 = $ys[pos: $i];\n");
        for (j, s) in self.steps.iter().enumerate() {
            let t = Self::name(3 + j);
            let v = |i: usize| format!("${}", Self::name(i));
            let line = match *s {
                Step::Wrap(_, w, a, b) => format!("{} {w} {}", v(a), v(b)),
                Step::Bits(_, w, a, b) => format!("{} {w} {}", v(a), v(b)),
                Step::ByLit(_, w, a, d) => format!("{} {w} {d}", v(a)),
                Step::Shr(a, c) => format!("{} shr {c}", v(a)),
                Step::Lit(n) => n.to_string(),
            };
            body.push_str(&format!("        let ${t}: u32 = {line};\n"));
        }
        body.push_str(&format!("        std::list.append(lend.mut $out, value: ${});\n", Self::name(self.last())));
        format!(
            "MAIN {{\n{}    let mut $out: std::list[u32] = std::list.empty;\n    for $i: u64 [from: 1, to: std::list.length(lend $xs), by: 1] {{\n{body}    }}\n    std::print.useful($out, newln);\n}}\n",
            self.lets()
        )
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
        o => return Outcome::Failed(format!("oracle, the loop: {}", crate::compare::oracle_text(&o))),
    };
    let kb = match kayo(t, &fb, &db) {
        Ok(out) => out,
        Err(e) => return Outcome::Failed(format!("kayo, the loop: {e}")),
    };
    let ka = match kayo(t, &fa, &da) {
        Ok(out) => out,
        Err(e) => return Outcome::Failed(format!("kayo, the kernel: {e}")),
    };
    let s = |v: &[u8]| String::from_utf8_lossy(v).trim_end().to_string();
    if o != kb {
        return Outcome::Differ(format!("the loop: oracle {}, kayo {}", s(&o), s(&kb)));
    }
    if ka != o {
        return Outcome::Differ(format!("the kernel on the GPU {}, the loop {}", s(&ka), s(&o)));
    }
    Outcome::Same
}
