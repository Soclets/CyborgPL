//! `inline.MSL` (owner, 2026-10-01, 2026-10-22; `design/gpu.md`): Metal's
//! own code, handed to Apple's compiler unchanged at build time. What kayo
//! reads of it is only what it must: the one `kernel` the block runs, the
//! files it `#include`s, and the Metal declarations of the `layout.C`
//! structs it binds, which kayo writes at the top so they are written once,
//! in CyborgPL. The library Apple's compiler makes is carried in the
//! program, and the runtime runs it on the GPU where the block stands.

use std::path::{Path, PathBuf};

use crate::types::{BinW, IntW, StructInfo, Ty};

/// The Metal text with its comments blanked out, so a `kernel` in a comment
/// is not one; every other byte stays where it was, so an offset in it is
/// one in the block.
fn uncommented(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let blank = |out: &mut String, c: char| {
        if c == '\n' {
            out.push('\n');
        } else {
            out.extend(std::iter::repeat_n(' ', c.len_utf8()));
        }
    };
    let mut cs = text.chars().peekable();
    while let Some(c) = cs.next() {
        match (c, cs.peek()) {
            ('/', Some('/')) => {
                out.push(' ');
                for d in cs.by_ref() {
                    blank(&mut out, d);
                    if d == '\n' {
                        break;
                    }
                }
            }
            ('/', Some('*')) => {
                out.push(' ');
                let mut last = ' ';
                for d in cs.by_ref() {
                    blank(&mut out, d);
                    if last == '*' && d == '/' {
                        break;
                    }
                    last = d;
                }
            }
            ('"', _) => {
                out.push('"');
                for d in cs.by_ref() {
                    out.push(d);
                    if d == '"' {
                        break;
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// The names of the `kernel` functions the block declares.
pub fn kernels(text: &str) -> Vec<String> {
    let code = uncommented(text);
    let words: Vec<&str> = code.split(|c: char| !(c.is_alphanumeric() || c == '_')).filter(|w| !w.is_empty()).collect();
    let mut out = Vec::new();
    for (i, w) in words.iter().enumerate() {
        // `kernel void twice(`: the name is the word after the return type.
        if *w == "kernel" {
            if let Some(name) = words.get(i + 2) {
                out.push(name.to_string());
            }
        }
    }
    out
}

/// The kernel's parameters bound to a `[[buffer(N)]]`: each one's slot,
/// where it starts in the block (a byte offset), and whether it is read
/// only — `const device …`, `device const …` or `constant …`.
pub fn buffer_params(text: &str, kernel: &str) -> Vec<(u32, usize, bool)> {
    let code = uncommented(text);
    // `kernel void name(`: the list after the name.
    let Some(open) = code.match_indices("kernel").find_map(|(at, _)| {
        let rest = &code[at..];
        let name_at = rest.find(kernel)?;
        let after = rest[name_at + kernel.len()..].trim_start();
        after.starts_with('(').then(|| at + name_at + kernel.len() + (rest[name_at + kernel.len()..].len() - after.len()))
    }) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = open + 1;
    for (i, c) in code[open..].char_indices().map(|(i, c)| (i + open, c)) {
        match c {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth -= 1,
            _ => {}
        }
        let ends = (c == ',' && depth == 1) || depth == 0;
        if ends && i > open {
            let param = &code[start..i];
            let lead = param.len() - param.trim_start().len();
            let squeezed: String = param.chars().filter(|c| !c.is_whitespace()).collect();
            if let Some(n) = squeezed.split("[[buffer(").nth(1).and_then(|r| r.split(')').next()).and_then(|n| n.parse::<u32>().ok()) {
                let ty = param.split(['*', '&']).next().unwrap_or("");
                let read_only = ty.split_whitespace().any(|w| w == "const" || w == "constant");
                out.push((n, start + lead, read_only));
            }
            start = i + 1;
            if depth == 0 {
                break;
            }
        }
    }
    out
}

/// The files the block `#include`s, each as written, with the line it is on
/// (from 1, within the block).
pub fn includes(text: &str) -> Vec<(String, usize)> {
    let code = uncommented(text);
    code.lines().enumerate().filter_map(|(n, l)| {
        let rest = l.trim().strip_prefix("#include")?.trim();
        let path = rest.strip_prefix('"')?.split('"').next()?;
        Some((path.to_string(), n + 1))
    }).collect()
}

/// An included path inside the kit's folder, lexically: `..` that climbs
/// out of it is refused, the file is not read.
pub fn inside_kit(root: &Path, path: &str) -> Option<PathBuf> {
    let mut depth = 0i32;
    for part in Path::new(path).components() {
        match part {
            std::path::Component::ParentDir => depth -= 1,
            std::path::Component::Normal(_) => depth += 1,
            std::path::Component::CurDir => {}
            _ => return None,
        }
        if depth < 0 {
            return None;
        }
    }
    Some(root.join(path))
}

/// What a type is called in Metal, for one that binds: Metal's scalars and
/// the `layout.C` structs. `None` for what has no Metal form (a 64-bit bin,
/// a deci, text).
pub fn metal_type(ty: &Ty, structs: &[StructInfo]) -> Option<String> {
    Some(match ty.base() {
        Ty::Bool => "bool".into(),
        Ty::Int(IntW::Fixed { signed, bits }) => match (signed, bits & !crate::types::EXACT_FLAG) {
            (true, 8) => "char",
            (true, 16) => "short",
            (true, 32) => "int",
            (true, 64) => "long",
            (false, 8) => "uchar",
            (false, 16) => "ushort",
            (false, 32) => "uint",
            (false, 64) => "ulong",
            _ => return None,
        }.into(),
        Ty::Bin(BinW::Fixed(16)) => "half".into(),
        Ty::Bin(BinW::Fixed(32)) => "float".into(),
        Ty::Struct { id, args } if args.is_empty() && structs[id.0 as usize].layout_c && !structs[id.0 as usize].opaque && !structs[id.0 as usize].union => structs[id.0 as usize].name.clone(),
        _ => return None,
    })
}

/// The Metal declarations of these `layout.C` structs, each after the ones
/// its fields hold, so Metal reads them in order.
pub fn declarations(roots: &[usize], structs: &[StructInfo]) -> Result<String, String> {
    fn visit(i: usize, structs: &[StructInfo], seen: &mut Vec<usize>, out: &mut String) -> Result<(), String> {
        if seen.contains(&i) {
            return Ok(());
        }
        seen.push(i);
        let s = &structs[i];
        let mut body = String::new();
        for f in &s.fields {
            if let Ty::Struct { id, .. } = f.ty.base() {
                visit(id.0 as usize, structs, seen, out)?;
            }
            let Some(t) = metal_type(&f.ty, structs) else {
                return Err(format!("field `{}` of `{}`", f.name, s.name));
            };
            body.push_str(&format!(" {} {};", t, f.name));
        }
        out.push_str(&format!("struct {} {{{} }};\n", s.name, body));
        Ok(())
    }
    let mut out = String::new();
    let mut seen = Vec::new();
    for &r in roots {
        visit(r, structs, &mut seen, &mut out)?;
    }
    Ok(out)
}

/// Apple's compiler, at build time: the source to a Metal library, the
/// kit's root the place `#include` paths start. The error, when it refuses,
/// is its first `error:` line, its line number counted within `source`.
/// `precise` (owner, 2026-10-25: `inline.MSL.precise`): safe math and no
/// contraction, `-fmetal-math-mode=safe -ffp-contract=off`; `wrap`
/// (`inline.MSL.wrap`): signed arithmetic wraps, `-fwrapv`. Without them,
/// Apple's defaults: fast math, signed overflow undefined.
pub fn compile(source: &str, kit_root: &Path, precise: bool, wrap: bool) -> Result<Vec<u8>, (Option<usize>, String)> {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!("kayo-msl-{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    std::fs::create_dir_all(&dir).map_err(|e| (None, format!("cannot make {}: {}", dir.display(), e)))?;
    let src = dir.join("block.metal");
    let air = dir.join("block.air");
    let lib = dir.join("block.metallib");
    std::fs::write(&src, source).map_err(|e| (None, format!("cannot write {}: {}", src.display(), e)))?;
    let run = |args: &[&std::ffi::OsStr]| std::process::Command::new("xcrun").args(["-sdk", "macosx"]).args(args).output();
    let mut args: Vec<&std::ffi::OsStr> = vec!["metal".as_ref(), "-c".as_ref(), src.as_os_str(), "-o".as_ref(), air.as_os_str(), "-I".as_ref(), kit_root.as_os_str()];
    if precise {
        args.extend::<[&std::ffi::OsStr; 2]>(["-fmetal-math-mode=safe".as_ref(), "-ffp-contract=off".as_ref()]);
    }
    if wrap {
        args.push("-fwrapv".as_ref());
    }
    let out = run(&args)
        .map_err(|e| (None, format!("cannot run Apple's Metal compiler (`xcrun metal`): {}", e)))?;
    if !out.status.success() {
        let text = String::from_utf8_lossy(&out.stderr);
        let line = text.lines().find(|l| l.contains("error:")).unwrap_or_else(|| text.lines().next().unwrap_or("it refused the block"));
        // `…/block.metal:12:5: error: …`: the line, and the message after it.
        let at = line.split_once("block.metal:").and_then(|(_, r)| r.split(':').next()).and_then(|n| n.parse::<usize>().ok());
        let said = line.split_once("error:").map(|(_, m)| m.trim().to_string()).unwrap_or_else(|| line.trim().to_string());
        let _ = std::fs::remove_dir_all(&dir);
        return Err((at, said));
    }
    let out = run(&["metallib".as_ref(), air.as_os_str(), "-o".as_ref(), lib.as_os_str()]).map_err(|e| (None, format!("cannot run `xcrun metallib`: {}", e)))?;
    if !out.status.success() {
        let _ = std::fs::remove_dir_all(&dir);
        return Err((None, String::from_utf8_lossy(&out.stderr).trim().to_string()));
    }
    let bytes = std::fs::read(&lib).map_err(|e| (None, format!("cannot read {}: {}", lib.display(), e)));
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}
