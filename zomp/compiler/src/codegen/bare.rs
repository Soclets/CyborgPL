//! A program with no operating system under it (owner, 2026-10-09:
//! `aarch64-none`): built on another machine, linked with lld's ELF
//! linker against a linker script, started by an entry Synapse writes, and
//! run on the bare chip (tested in QEMU's `virt` machine).
//!
//! Nothing here comes from a C library: the few memory routines LLVM's code
//! calls are written below, each in its own section so the link keeps only
//! those used. Axon, the runtime, is built for the chip without its heap,
//! tasks and channels (`cargo build -p zomp-runtime --target
//! aarch64-unknown-none`); a program that needs those is refused before
//! linking, naming what needs it.

use std::path::Path;

use inkwell::module::Module;

/// The memory routines LLVM may call for a copy, a fill or a comparison;
/// C's names, since that is what LLVM calls. Byte at a time.
pub const MEMORY_ASM: &str = r#"
.section .text.memcpy,"ax",@progbits
.globl memcpy
.type memcpy, %function
.p2align 2
memcpy:
    mov x3, x0
1:  cbz x2, 2f
    ldrb w4, [x1], #1
    strb w4, [x3], #1
    sub x2, x2, #1
    b 1b
2:  ret
.size memcpy, .-memcpy

.section .text.memmove,"ax",@progbits
.globl memmove
.type memmove, %function
.p2align 2
memmove:
    cmp x0, x1
    b.ls memcpy
    add x3, x1, x2
    cmp x0, x3
    b.hs memcpy
    add x4, x0, x2
1:  cbz x2, 2f
    ldrb w5, [x3, #-1]!
    strb w5, [x4, #-1]!
    sub x2, x2, #1
    b 1b
2:  ret
.size memmove, .-memmove

.section .text.memset,"ax",@progbits
.globl memset
.type memset, %function
.p2align 2
memset:
    mov x3, x0
1:  cbz x2, 2f
    strb w1, [x3], #1
    sub x2, x2, #1
    b 1b
2:  ret
.size memset, .-memset

.section .text.memcmp,"ax",@progbits
.globl memcmp
.globl bcmp
.type memcmp, %function
.type bcmp, %function
.p2align 2
memcmp:
bcmp:
1:  cbz x2, 2f
    ldrb w3, [x0], #1
    ldrb w4, [x1], #1
    subs w3, w3, w4
    b.ne 3f
    sub x2, x2, #1
    b 1b
2:  mov w0, #0
    ret
3:  mov w0, w3
    ret
.size memcmp, .-memcmp
.size bcmp, .-bcmp
"#;

/// What a function with no body left after LLVM's passes is, when the
/// program is for a bare chip: nothing there will define it.
pub fn missing(module: &Module) -> Vec<String> {
    const OURS: [&str; 5] = ["memcpy", "memmove", "memset", "memcmp", "bcmp"];
    let mut out = Vec::new();
    let mut f = module.get_first_function();
    while let Some(func) = f {
        let name = func.get_name().to_string_lossy().into_owned();
        let axon = name.starts_with("cyborg_") && !needs_os(&name);
        if func.count_basic_blocks() == 0 && !name.starts_with("llvm.") && !OURS.contains(&name.as_str()) && !axon && inkwell::values::BasicValue::get_first_use(&func.as_global_value().as_pointer_value()).is_some() {
            out.push(name);
        }
        f = func.get_next_function();
    }
    out
}

/// Whether Axon's function `name` stands on the heap, threads or the OS,
/// which a chip with no OS does not have: memory, text and lists that
/// grow, maps, tasks and channels.
fn needs_os(name: &str) -> bool {
    let n = name.trim_start_matches("cyborg_");
    ["alloc", "free", "realloc", "copy", "move", "spawn", "run", "whichever", "machine_has", "check_failed", "proof_failed"].contains(&n)
        || ["str_", "map_", "chan_", "par_", "group_", "task_", "must_", "test_", "alloc"].iter().any(|p| n.starts_with(p))
        || n.ends_with("_to_text")
        || n.ends_with("_text_useful")
}

/// The sentence for one of them.
pub fn why_missing(name: &str) -> String {
    if name.starts_with("cyborg_") {
        format!("not yet on `{}`: this program needs the runtime's heap, threads or OS (`{}`): text and lists that grow, maps, tasks and channels are not built for a chip with no OS", crate::target::name(), name)
    } else {
        format!("not yet on `{}`: `{}` is not defined anywhere — there is no C library under a program with no OS", crate::target::name(), name)
    }
}

/// The entry: the first code the CPU runs. Only the first core goes on
/// (any other sleeps); the FP and SIMD unit is turned on, or the first
/// float or vector instruction would trap; an exception — a failed check's
/// `brk` — stops in a table that waits forever (the table is the start of
/// the image, 2 KiB aligned as `VBAR_EL1` asks, and the entry sits in the
/// first slot's 128 bytes, which only an exception taken on `SP_EL0` would
/// reach, and the program runs on `SP_EL1`: no padding before the table,
/// which after the code cost up to 2 KiB), `ELR_EL1` holding the
/// trap's address and `ESR_EL1` saying it was one; the stack is set and
/// zeroed memory cleared; then `MAIN` runs through `main`, and when it ends
/// the CPU sleeps there.
pub const ENTRY_ASM: &str = r#"
.section .text._start,"ax",@progbits
.globl _start
.type _start, %function
.p2align 11
cy_vectors:
_start:
    mrs x0, mpidr_el1
    and x0, x0, #0xff
    cbnz x0, 9f
    mov x0, #(3 << 20)
    msr cpacr_el1, x0
    isb
    adrp x0, cy_vectors
    add x0, x0, :lo12:cy_vectors
    msr vbar_el1, x0
    isb
    adrp x0, __stack_top
    add x0, x0, :lo12:__stack_top
    mov sp, x0
    adrp x0, __bss_start
    add x0, x0, :lo12:__bss_start
    adrp x1, __bss_end
    add x1, x1, :lo12:__bss_end
1:  cmp x0, x1
    b.hs 2f
    str xzr, [x0], #8
    b 1b
2:  mov w0, #0
    mov x1, #0
    bl main
9:  wfe
    b 9b
.size _start, .-_start
.rept 15
.p2align 7
1:  wfe
    b 1b
.endr
.size cy_vectors, .-cy_vectors
"#;

extern "C" {
    fn cca_lld_link(argc: std::os::raw::c_int, argv: *const *const std::os::raw::c_char, said: *mut std::os::raw::c_char, said_len: usize) -> std::os::raw::c_int;
}

/// Link with lld's ELF linker against the Neuron's own linker script
/// (owner, 2026-10-09: "`c` now"): where the program sits is the board's,
/// so Synapse has no layout of its own.
/// Axon built for the bare chip, beside the compiler as cargo puts it.
fn runtime() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let triple = crate::target::bare_triple();
    for dir in exe.parent()?.ancestors().take(4) {
        for profile in ["release", "debug"] {
            let p = dir.join(&triple).join(profile).join("libzomp_runtime.a");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

pub fn link(obj: &Path, script: &Path, written: &str, out: &Path) -> Result<(), String> {
    let axon = runtime().ok_or_else(|| format!("the runtime is not built for `{}`: `cargo build -p zomp-runtime --release --target {}`", crate::target::name(), crate::target::bare_triple()))?;
    let args: Vec<String> = vec![
        "ld.lld".into(),
        "-nostdlib".into(),
        "--gc-sections".into(),
        "-static".into(),
        // No page alignment between sections: nothing pages a bare chip.
        "--nmagic".into(),
        "-T".into(),
        script.to_string_lossy().into_owned(),
        "-o".into(),
        out.to_string_lossy().into_owned(),
        obj.to_string_lossy().into_owned(),
        axon.to_string_lossy().into_owned(),
    ];
    let c: Vec<std::ffi::CString> = args.iter().map(|a| std::ffi::CString::new(a.as_str()).map_err(|_| format!("a path holds a NUL byte: {}", a))).collect::<Result<_, _>>()?;
    let argv: Vec<*const std::os::raw::c_char> = c.iter().map(|a| a.as_ptr()).collect();
    let mut said = vec![0 as std::os::raw::c_char; 16 * 1024];
    let code = unsafe { cca_lld_link(argv.len() as std::os::raw::c_int, argv.as_ptr(), said.as_mut_ptr(), said.len()) };
    if code == 0 {
        return Ok(());
    }
    let said = unsafe { std::ffi::CStr::from_ptr(said.as_ptr()) }.to_string_lossy().trim().to_string();
    // The three addresses the entry reads are the script's to define
    // (owner, 2026-10-09: "Whatever's accurate", the C world's names).
    // Something kept still stands on what a bare chip lacks.
    for os in ["malloc", "free", "realloc", "calloc", "write", "exit", "pthread_create", "mmap", "sysctlbyname"] {
        if said.contains(&format!("undefined symbol: {}\n", os)) || said.ends_with(&format!("undefined symbol: {}", os)) {
            return Err(format!("not yet on `{}`: something in this program needs `{}`, which a chip with no OS does not have", crate::target::name(), os));
        }
    }
    for (name, what) in [("__stack_top", "where the stack starts"), ("__bss_start", "where the memory zeroed before MAIN starts"), ("__bss_end", "where the memory zeroed before MAIN ends")] {
        if said.contains(&format!("undefined symbol: {}", name)) {
            return Err(format!("the linker script `{}` does not define `{}`, {}", written, name, what));
        }
    }
    Err(if said.is_empty() { "the linker (lld) refused the program".into() } else { format!("the linker (lld) refused the program:\n{}", said) })
}
