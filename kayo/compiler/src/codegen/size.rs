//! The size report every build ends with (owner, 2026-10-08: "After
//! finished compiling, it will report size with a breakdown"). The
//! breakdown is design claude's reading, not asked: by Neuron, by
//! function, and the copies each generic produced.
//!
//! The program's own share is measured in the object file Synapse wrote,
//! before linking: each function's code runs from its symbol to the next
//! one in its section. A function LLVM inlined into every caller has no
//! symbol left and takes no room of its own. The rest of the file on disk
//! is Axon, the runtime, what the linker adds, and the file's layout.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::Path;

use llvm_sys::core::{LLVMCreateMemoryBufferWithContentsOfFile, LLVMDisposeMessage};
use llvm_sys::object::*;

use crate::mir::Program;

/// One symbol in the object: its name, section and place.
struct Sym {
    name: String,
    section: usize,
    addr: u64,
    /// ELF's own size for it; 0 where the format keeps none (Mach-O).
    size: u64,
}

/// The object's code, by symbol, and its data, in bytes.
fn measure(obj: &Path) -> Result<(HashMap<String, u64>, u64), String> {
    let path = CString::new(obj.to_string_lossy().as_bytes()).map_err(|_| "the object's path holds a NUL byte".to_string())?;
    unsafe {
        let mut buf = std::ptr::null_mut();
        let mut msg = std::ptr::null_mut();
        if LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut buf, &mut msg) != 0 {
            let said = CStr::from_ptr(msg).to_string_lossy().into_owned();
            LLVMDisposeMessage(msg);
            return Err(said);
        }
        let mut msg = std::ptr::null_mut();
        let bin = LLVMCreateBinary(buf, std::ptr::null_mut(), &mut msg);
        if bin.is_null() {
            let said = if msg.is_null() { "not an object file".to_string() } else { CStr::from_ptr(msg).to_string_lossy().into_owned() };
            if !msg.is_null() {
                LLVMDisposeMessage(msg);
            }
            llvm_sys::core::LLVMDisposeMemoryBuffer(buf);
            return Err(said);
        }
        // ELF keeps each symbol's size; Mach-O does not (LLVM's C API hands
        // back the address there).
        let elf = matches!(LLVMBinaryGetType(bin), LLVMBinaryType::LLVMBinaryTypeELF32L | LLVMBinaryType::LLVMBinaryTypeELF32B | LLVMBinaryType::LLVMBinaryTypeELF64L | LLVMBinaryType::LLVMBinaryTypeELF64B);
        // Every section: its name, start and size.
        let mut sections: Vec<(String, u64, u64)> = Vec::new();
        let si = LLVMObjectFileCopySectionIterator(bin);
        while LLVMObjectFileIsSectionIteratorAtEnd(bin, si) == 0 {
            let name = LLVMGetSectionName(si);
            let name = if name.is_null() { String::new() } else { CStr::from_ptr(name).to_string_lossy().into_owned() };
            sections.push((name, LLVMGetSectionAddress(si), LLVMGetSectionSize(si)));
            LLVMMoveToNextSection(si);
        }
        LLVMDisposeSectionIterator(si);
        // Every named symbol, with the section that holds it (found by its
        // start, which no two sections share).
        let mut syms: Vec<Sym> = Vec::new();
        let yi = LLVMObjectFileCopySymbolIterator(bin);
        let at = LLVMObjectFileCopySectionIterator(bin);
        while LLVMObjectFileIsSymbolIteratorAtEnd(bin, yi) == 0 {
            let name = LLVMGetSymbolName(yi);
            let name = if name.is_null() { String::new() } else { CStr::from_ptr(name).to_string_lossy().into_owned() };
            LLVMMoveToContainingSection(at, yi);
            if LLVMObjectFileIsSectionIteratorAtEnd(bin, at) == 0 {
                let start = LLVMGetSectionAddress(at);
                let size = LLVMGetSectionSize(at);
                if let Some(section) = sections.iter().position(|s| s.1 == start && s.2 == size) {
                    syms.push(Sym { name, section, addr: LLVMGetSymbolAddress(yi), size: if elf { LLVMGetSymbolSize(yi) } else { 0 } });
                }
            }
            LLVMMoveToNextSymbol(yi);
        }
        LLVMDisposeSectionIterator(at);
        LLVMDisposeSymbolIterator(yi);
        LLVMDisposeBinary(bin);
        let is_code = |n: &str| n == "__text" || n == ".text" || n.starts_with(".text.");
        let mut code: HashMap<String, u64> = HashMap::new();
        for (i, (name, start, size)) in sections.iter().enumerate() {
            if !is_code(name) {
                continue;
            }
            // Assembler-made labels (`ltmp0`, `l_…`, `.L…`), ARM's mapping
            // symbols (`$x`, `$d`) and sections' own names mark places, not
            // functions.
            let mut here: Vec<&Sym> = syms.iter().filter(|s| s.section == i && !s.name.starts_with("ltmp") && !s.name.starts_with("l_") && !s.name.starts_with('.') && !s.name.starts_with('$') && !s.name.is_empty()).collect();
            here.sort_by_key(|s| s.addr);
            for (k, s) in here.iter().enumerate() {
                let end = if s.size > 0 { s.addr + s.size } else { here[k + 1..].iter().map(|n| n.addr).find(|a| *a > s.addr).unwrap_or(start + size) };
                // Mach-O writes a C name with `_` in front; ELF does not.
                let name = s.name.strip_prefix('_').filter(|_| cfg!(target_os = "macos") && !crate::target::bare()).unwrap_or(&s.name).to_string();
                *code.entry(name).or_default() += end - s.addr;
            }
        }
        // Only what the program holds while it runs: constants, text,
        // variables; not the object's own bookkeeping (symbols, relocations,
        // unwind tables, debug information).
        let is_data = |n: &str| [".rodata", ".data", ".bss", ".tdata", ".tbss", "__const", "__cstring", "__data", "__bss", "__common", "__literal"].iter().any(|p| n.starts_with(p));
        let data = sections.iter().filter(|(n, _, _)| is_data(n)).map(|s| s.2).sum();
        Ok((code, data))
    }
}

/// `cy_12_name`: one of the program's functions, as `declare_all` names it.
fn is_function(symbol: &str) -> bool {
    let rest = symbol.strip_prefix("cy_main_").or_else(|| symbol.strip_prefix("cy_").and_then(|r| r.split_once('_')).map(|(n, _)| n));
    rest.is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn bytes(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!("{} byte{}", out, if n == 1 { "" } else { "s" })
}

/// The report, from the object written for `p` and the file linked from it.
pub fn report(p: &Program, obj: &Path, exe: &Path) -> Result<String, String> {
    // With no OS the linked file is all the program's, and only what the
    // link kept is in it; otherwise the object, before Axon joins it.
    let (code, data) = measure(if crate::target::bare() { exe } else { obj })?;
    let on_disk = std::fs::metadata(exe).map_err(|e| format!("cannot read {}: {}", exe.display(), e))?.len();
    let spell = |t: &crate::types::Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
    // Each function's line: its Neuron, the name it is known by, and the
    // generic it is a copy of.
    let mut rows: Vec<(String, String, Option<String>, u64)> = Vec::new();
    let mut inlined: Vec<String> = Vec::new();
    let mut own = 0u64;
    for (i, f) in p.functions.iter().enumerate() {
        let symbol = if p.main.is_some_and(|m| m.0 as usize == i) { format!("cy_main_{}", i) } else { format!("cy_{}_{}", i, super::sanitise(&f.name)) };
        // `MAIN`'s instance belongs to no declared function.
        let found = f.inst.and_then(|inst| p.instances.get(inst.0 as usize)).and_then(|inst| p.items.fns.get(inst.key.fn_id.0 as usize).map(|d| (inst, d)));
        let (neuron, name, generic) = match found {
            Some((inst, decl)) => {
                let root = p.items.kit_root(decl.module);
                let neuron = p.items.kit_powers.get(&root).filter(|l| !l.is_program).map(|l| format!("`{}::`", l.prefix)).unwrap_or_else(|| "this kit".into());
                if inst.key.blanks.is_empty() {
                    (neuron, f.name.clone(), None)
                } else {
                    // A blank a bare literal filled is built 64 bits wide, as
                    // the code generator builds it.
                    let built = |t: &crate::types::Ty| match t {
                        crate::types::Ty::Int(crate::types::IntW::Free(_)) => crate::types::Ty::int(true, 64),
                        crate::types::Ty::Bin(crate::types::BinW::Free(_)) => crate::types::Ty::Bin(crate::types::BinW::Fixed(64)),
                        other => other.clone(),
                    };
                    let args = inst.key.blanks.iter().map(|t| spell(&built(t))).collect::<Vec<_>>().join(", ");
                    (neuron, format!("{}[{}]", decl.name, args), Some(decl.name.clone()))
                }
            }
            None => ("this kit".into(), f.name.clone(), None),
        };
        let Some(&size) = code.get(&symbol) else {
            inlined.push(if neuron == "this kit" { name } else { format!("{}{}", neuron.trim_matches('`'), name) });
            continue;
        };
        own += size;
        rows.push((neuron, name, generic, size));
    }
    // What Synapse writes around the program: the entry that runs `MAIN`
    // (often with `MAIN` inlined into it), and a test run's table.
    // Synapse's own code around the program: the entry and, with no OS,
    // `_start`, its vector table and the memory routines. Anything else in
    // a linked file is Axon's, or Rust's `core` under it: one line.
    const GLUE: [&str; 8] = ["main", "_start", "cy_vectors", "memcpy", "memmove", "memset", "memcmp", "bcmp"];
    let mut glue: Vec<(&String, &u64)> = code.iter().filter(|(n, _)| !is_function(n) && (GLUE.contains(&n.as_str()) || n.starts_with("cy_test"))).collect();
    glue.sort();
    // With no OS, `_start` sits in the vector table's first slot: counted
    // once, in the table's row.
    let in_table = code.contains_key("cy_vectors");
    for (n, s) in glue {
        if n == "_start" && in_table {
            continue;
        }
        let name = match n.as_str() {
            "main" => "the entry, `main`, which runs MAIN".to_string(),
            "cy_vectors" => "the vector table, `_start` in its first slot, KAYO\\n's".to_string(),
            _ => format!("`{}`, KAYO\\n's", n),
        };
        own += *s;
        rows.push(("this kit".into(), name, None, *s));
    }
    let axon: Vec<u64> = code.iter().filter(|(n, _)| !is_function(n) && !GLUE.contains(&n.as_str()) && !n.starts_with("cy_test")).map(|(_, s)| *s).collect();
    let mut out = if crate::target::bare() {
        format!(
            "size: {} on disk; the program's own code is {}, the runtime's {} in {} function{}, and the data {}, the runtime's tables included; the rest is the file's own layout\n",
            bytes(on_disk),
            bytes(own),
            bytes(axon.iter().sum()),
            axon.len(),
            if axon.len() == 1 { "" } else { "s" },
            bytes(data)
        )
    } else {
        format!("size: {} on disk; the program's own code is {} and its data {}; the rest is the runtime and what the linker adds\n", bytes(on_disk), bytes(own), bytes(data))
    };
    let mut neurons: Vec<String> = rows.iter().map(|r| r.0.clone()).collect();
    neurons.sort();
    neurons.dedup();
    // This Neuron first, then each dependency by its prefix.
    neurons.sort_by_key(|n| n != "this kit");
    let width = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(0);
    for n in &neurons {
        let mut mine: Vec<&(String, String, Option<String>, u64)> = rows.iter().filter(|r| &r.0 == n).collect();
        mine.sort_by(|a, b| b.3.cmp(&a.3).then(a.1.cmp(&b.1)));
        if neurons.len() > 1 {
            out.push_str(&format!("  {}: {}\n", n, bytes(mine.iter().map(|r| r.3).sum())));
        }
        let indent = if neurons.len() > 1 { "    " } else { "  " };
        for (_, name, generic, size) in mine {
            let copy = match generic {
                Some(g) => {
                    let copies = rows.iter().filter(|r| r.2.as_deref() == Some(g.as_str())).count();
                    format!("   a copy of the generic `{}` ({} in all)", g, copies)
                }
                None => String::new(),
            };
            out.push_str(&format!("{}{:<width$}  {:>8}{}\n", indent, name, size, copy, width = width));
        }
    }
    if !inlined.is_empty() {
        inlined.sort();
        let one = inlined.len() == 1;
        out.push_str(&format!("  inlined into {} callers, taking no room of {} own: {}\n", if one { "its" } else { "their" }, if one { "its" } else { "their" }, inlined.join(", ")));
    }
    Ok(out)
}
