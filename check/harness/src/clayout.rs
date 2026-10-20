//! `layout.C` against clang (owner, 2026-10-19: "(a)"): a random C-shaped
//! struct or union, bit-fields included, built once by kayo and once as C by
//! clang. Each side prints its size and alignment, then the object's bytes
//! with one field set to all ones and the rest zero, field by field. Sizes
//! and alignments must be equal, and each dump equal under the mask of bits
//! clang's fields cover (padding may hold anything). kayo's bytes are read
//! with C's own `write`, given the object's address.

use crate::engines::{self, Tools};
use check_gen::rng::Rng;
use std::path::Path;
use std::process::Command;

const CLANG: &str = "/usr/bin/clang";

#[derive(Clone, Copy)]
enum Kind {
    /// a whole number of this many bits, signed or not
    Int(u32, bool),
    Bool,
    /// a bit-field of N bits: uint8 or int8 (signed), or bool[bits 1]
    Bits(u32, bool),
    BoolBit,
}

struct Field {
    name: String,
    kind: Kind,
}

pub struct Layout {
    union: bool,
    fields: Vec<Field>,
}

impl Layout {
    pub fn random(seed: u64) -> Layout {
        let mut r = Rng::new(seed);
        let union = r.chance(25);
        let n = if union { r.range(2, 4) } else { r.range(1, 8) } as usize;
        let mut fields = vec![];
        for i in 0..n {
            let kind = match r.below(if union { 2 } else { 4 }) {
                0 => Kind::Int(*r.pick(&[8u32, 16, 32, 64]), r.chance(50)),
                1 => Kind::Bool,
                2 => Kind::Bits(r.range(1, 8) as u32, r.chance(40)),
                _ => Kind::BoolBit,
            };
            // a signed bit-field of one bit holds 0 and -1; keep it two or more
            let kind = match kind {
                Kind::Bits(1, true) => Kind::Bits(2, true),
                k => k,
            };
            fields.push(Field { name: format!("f{i}"), kind });
        }
        Layout { union, fields }
    }

    fn cy_type(k: Kind) -> String {
        match k {
            Kind::Int(b, s) => format!("{}{b}", if s { "i" } else { "u" }),
            Kind::Bool => "bool".into(),
            Kind::Bits(n, s) => format!("{}8[bits: {n}]", if s { "i" } else { "u" }),
            Kind::BoolBit => "bool[bits: 1]".into(),
        }
    }

    fn c_type(k: Kind) -> String {
        match k {
            Kind::Int(b, s) => format!("{}int{b}_t", if s { "" } else { "u" }),
            Kind::Bool => "bool".into(),
            Kind::Bits(_, s) => format!("{}int8_t", if s { "" } else { "u" }),
            Kind::BoolBit => "bool".into(),
        }
    }

    fn ones(k: Kind) -> String {
        match k {
            Kind::Int(_, true) | Kind::Bits(_, true) => "(-1)".into(),
            Kind::Int(b, false) => ((1u128 << b) - 1).to_string(),
            Kind::Bits(n, false) => ((1u32 << n) - 1).to_string(),
            Kind::Bool | Kind::BoolBit => "true".into(),
        }
    }

    fn zero(k: Kind) -> &'static str {
        match k {
            Kind::Bool | Kind::BoolBit => "false",
            _ => "0",
        }
    }

    pub fn cy(&self) -> String {
        let decl = self.fields.iter().map(|f| format!("{}: {}", f.name, Self::cy_type(f.kind))).collect::<Vec<_>>().join(", ");
        let mut s = format!("layout.C {} L {{ {decl} }}\n", if self.union { "union" } else { "struct" });
        s.push_str("extern.C fn dump_l = C.write ['fd': i32, 'buf': ptr[L], 'len': u64] -> i64;\n\nMAIN {\n");
        s.push_str("    std::print.useful[\"size \", size.of[L], \" align \", align.of[L], newln];\n");
        for (i, f) in self.fields.iter().enumerate() {
            let init = if self.union {
                format!("{} = {}", f.name, Self::ones(f.kind))
            } else {
                self.fields.iter().enumerate().map(|(j, g)| format!("{} = {}", g.name, if i == j { Self::ones(g.kind) } else { Self::zero(g.kind).to_string() })).collect::<Vec<_>>().join(", ")
            };
            s.push_str(&format!("    let 'v{i}' := L[{init}];\n    no_check dump_l[1, lend 'v{i}', size.of[L]];\n"));
        }
        s.push_str("}\n");
        s
    }

    pub fn c(&self) -> String {
        let mut s = String::from("#include <stdio.h>\n#include <string.h>\n#include <stdint.h>\n#include <stdbool.h>\n#include <stdalign.h>\n");
        s.push_str(&format!("{} L {{ ", if self.union { "union" } else { "struct" }));
        for f in &self.fields {
            let bits = match f.kind {
                Kind::Bits(n, _) => format!(" : {n}"),
                Kind::BoolBit => " : 1".into(),
                _ => String::new(),
            };
            s.push_str(&format!("{} {}{bits}; ", Self::c_type(f.kind), f.name));
        }
        let tag = if self.union { "union" } else { "struct" };
        s.push_str(&format!("}};\nint main(void) {{\n    printf(\"size %zu align %zu\\n\", sizeof({tag} L), alignof({tag} L));\n    fflush(stdout);\n    {tag} L v;\n"));
        for f in &self.fields {
            s.push_str(&format!("    memset(&v, 0, sizeof v); v.{} = {}; fwrite(&v, 1, sizeof v, stdout);\n", f.name, Self::ones(f.kind)));
        }
        s.push_str("    return 0;\n}\n");
        s
    }
}

pub enum Outcome {
    Same,
    Differ(String),
    /// kayo or clang did not build or run it: the reason
    Failed(String),
}

/// The text line, then the dumps of `size` bytes each.
fn split(out: &[u8]) -> Option<(String, Vec<Vec<u8>>)> {
    let nl = out.iter().position(|b| *b == b'\n')?;
    let head = String::from_utf8_lossy(&out[..nl]).to_string();
    let size: usize = head.split_whitespace().nth(1)?.parse().ok()?;
    let rest = &out[nl + 1..];
    if size == 0 || rest.len() % size != 0 {
        return None;
    }
    Some((head, rest.chunks(size).map(|c| c.to_vec()).collect()))
}

pub fn check(t: &Tools, l: &Layout, dir: &Path) -> Outcome {
    let _ = std::fs::create_dir_all(dir);
    let (kf, cf) = (dir.join("k.cy"), dir.join("c.c"));
    let (kb, cb) = (dir.join("k"), dir.join("c"));
    let _ = std::fs::remove_file(&kb);
    let _ = std::fs::remove_file(&cb);
    if std::fs::write(&kf, l.cy()).is_err() || std::fs::write(&cf, l.c()).is_err() {
        return Outcome::Failed("cannot write the files".into());
    }
    let r = engines::run(Command::new(&t.cca).arg("build").arg(&kf).arg("-o").arg(&kb), dir, &[], false, engines::build_limit());
    if r.code != Some(0) || !kb.exists() {
        return Outcome::Failed(format!("kayo: {}", r.stderr.lines().find(|l| l.starts_with("error")).unwrap_or("did not build")));
    }
    let r = engines::run(Command::new(CLANG).arg("-O0").arg(&cf).arg("-o").arg(&cb), dir, &[], false, engines::build_limit());
    if r.code != Some(0) || !cb.exists() {
        return Outcome::Failed(format!("clang: {}", r.stderr.lines().next().unwrap_or("did not build")));
    }
    let (k, c) = (engines::run(&mut Command::new(&kb), dir, &[], false, engines::RUN_LIMIT), engines::run(&mut Command::new(&cb), dir, &[], false, engines::RUN_LIMIT));
    let (Some((kh, kd)), Some((ch, cd))) = (split(&k.stdout), split(&c.stdout)) else {
        return Outcome::Failed("an output could not be read".into());
    };
    if kh != ch {
        return Outcome::Differ(format!("kayo `{kh}`, clang `{ch}`"));
    }
    if kd.len() != cd.len() {
        return Outcome::Differ(format!("kayo wrote {} dumps, clang {}", kd.len(), cd.len()));
    }
    // the bits any field covers, as clang lays them out
    let mut mask = vec![0u8; cd[0].len()];
    for d in &cd {
        for (m, b) in mask.iter_mut().zip(d) {
            *m |= b;
        }
    }
    for (i, (x, y)) in kd.iter().zip(&cd).enumerate() {
        let xm: Vec<u8> = x.iter().zip(&mask).map(|(a, m)| a & m).collect();
        let ym: Vec<u8> = y.iter().zip(&mask).map(|(a, m)| a & m).collect();
        if xm != ym {
            let hex = |v: &[u8]| v.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
            return Outcome::Differ(format!("field {} ({}): kayo {}, clang {}", l.fields[i].name, Layout::cy_type(l.fields[i].kind), hex(&xm), hex(&ym)));
        }
    }
    Outcome::Same
}
