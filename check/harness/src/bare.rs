//! A chip with no operating system (`aarch64-none`, owner 2026-10-09; the
//! oracle reads a target's `layout`, `output` and `exit` since 29b59ddc):
//! each generated program, written with no C and no heap (its printer through
//! `std::print`), is built by kayo as a kit for QEMU's `virt` board and booted
//! there, its prints going out of the board's serial port; the oracle runs the
//! same program as an ordinary file. What QEMU prints must be what the oracle
//! printed; where the oracle stopped at a check, its output must be how QEMU's
//! begins (kayo's stop message going out of the same port), and the board
//! freezes there (owner, 2026-10-08), so a run that does not end is right. MAIN's
//! end switches the board off through the target's `exit` (owner, 2026-10-26).
//!
//! The board is `examples/bare_station`'s (a PL011 serial port at 0x0900_0000,
//! power off by PSCI over `hvc`), its linker script the same.

use crate::engines::{self, Tools};
use check_gen::Program;
use std::path::Path;
use std::process::Command;

const QEMU: &str = "/opt/homebrew/bin/qemu-system-aarch64";

/// The board's two jobs, as `examples/bare_station` writes them.
const BOARD: &str = r#"
module board {
    # The serial port is a PL011 at 0x0900_0000: a byte written to its data register goes out.
    pub.kit fn write [$b: u8] {
        no_check inline.ASM {
            mov x9, #0x09000000
            strb w0, [x9]
        } with [$b -> x0, clobber x9];
    }

    # Power off through PSCI's SYSTEM_OFF (0x8400_0008), over `hvc`; QEMU ends with it.
    pub.kit fn off [$code: u8] {
        no_check inline.ASM {
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        } with [$code -> x1, clobber x0];
    }
}
"#;

/// QEMU's `virt` machine, as `examples/bare_station/boards/virt.ld`.
const VIRT_LD: &str = "ENTRY(_start)
SECTIONS {
    . = 0x40080000;
    .text : { KEEP(*(.text._start)) *(.text .text.*) }
    .rodata : ALIGN(16) { *(.rodata .rodata.*) }
    .data : ALIGN(16) { *(.data .data.*) }
    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.* COMMON)
        . = ALIGN(16);
        __bss_end = .;
    }
    .stack (NOLOAD) : ALIGN(16) {
        . += 0x10000;
        __stack_top = .;
    }
    /DISCARD/ : { *(.comment) *(.note .note.*) *(.eh_frame) }
}
";

/// std from this clone, CNC for `layout.C` enums and WSMW for odd widths (the
/// generator's, design/cores.md ids), the powers the board needs.
fn manifest(std: &Path) -> String {
    format!(
        "my details:\nname = bare\nprefix = bare\nver = 0.1.0\n\nmy kits:\nstd {{\npath = {}\n}}\n\nmy limbs:\ncnc {{\nid = 04b99a7cbb141b84b9f2f38c085c124e2badc37d663f71c6deebdd6ac24e79b1\nver = 2026-10-10T16:42:00+07:00\n}}\nwsmw {{\nid = 0fb49902b57b8bc0b72caf424d80e17d035038e2cac58292d92f18b792d205a0\nver = 2026-10-07T20:22:24+07:00\n}}\n\nmy powers:\nvouch {{\n}}\ninline_assembly {{\n}}\n\nmy targets:\naarch64-none {{\nlayout = boards/virt.ld\noutput = my.board.write\nexit = my.board.off\n}}\n",
        std.display()
    )
}

pub enum Outcome {
    Same,
    /// kayo says the program needs what a chip with no OS lacks yet (a heap,
    /// threads, an OS): coverage, not a finding
    NotYet(String),
    Differ(String),
    /// the oracle or kayo did not run it, or QEMU did not end: why
    Failed(String),
}

pub fn check(t: &Tools, p: &Program, dir: &Path) -> Outcome {
    // MAIN's end calls the target's `exit` with 0 (owner, 2026-10-26; kayo 78a41606)
    check_text(t, &p.render(), dir)
}

/// How long the board gets: a program here runs in well under a second, and a
/// failed check freezes the board (owner, 2026-10-08: "freeze there").
const BOARD_LIMIT: std::time::Duration = std::time::Duration::from_secs(5);

/// A program as text (a generated one, or one of `corpus-bare/`), its MAIN
/// ending in `std::exit`: on the oracle as a file, and in QEMU as a kit.
pub fn check_text(t: &Tools, text: &str, dir: &Path) -> Outcome {
    let kit = dir.join("kit");
    let _ = std::fs::create_dir_all(kit.join("boards"));
    let text = text.to_string();
    let (f, bin) = (dir.join("prog.cy"), dir.join("prog"));
    let _ = std::fs::remove_file(&bin);
    let Some(std) = t.cca.ancestors().nth(4).map(|r| r.join("std")) else {
        return Outcome::Failed("cannot find std/".into());
    };
    let wrote = std::fs::write(&f, &text).is_ok()
        && std::fs::write(kit.join("kit.cy"), format!("{text}{BOARD}")).is_ok()
        && std::fs::write(kit.join("boards/virt.ld"), VIRT_LD).is_ok()
        && std::fs::write(kit.join("CyJet.toml"), manifest(&std)).is_ok();
    if !wrote {
        return Outcome::Failed("cannot write the files".into());
    }
    let o = engines::oracle(t, &f, dir, &[]);
    let r = engines::run(Command::new(&t.cca).arg("build").arg(&kit).arg("-o").arg(&bin), dir, &[], false, engines::build_limit());
    if r.code != Some(0) || !bin.exists() {
        // a failure kayo sees while building, at the line the oracle stops at,
        // is the same answer, as in the host comparison
        if let Some(l) = r.stderr.lines().find(|l| l.contains("not yet on `aarch64-none`") || l.starts_with("error[not_built_yet]")) {
            return Outcome::NotYet(l.to_string());
        }
        // or before it: the oracle stopped earlier in the run, at a check kayo
        // keeps, and never reached what kayo sees fail every time (as compare.rs)
        let at = engines::kayo_line(&r.stderr);
        if let engines::Oracle::Stopped { line: Some(l), .. } = &o {
            if at.is_some_and(|k| k >= *l) {
                return Outcome::Same;
            }
        }
        let why = r.stderr.lines().find(|l| l.starts_with("error")).unwrap_or("did not build").to_string();
        return Outcome::Failed(format!("kayo: {why}"));
    }
    let q = engines::run(
        Command::new(QEMU).args(["-M", "virt", "-cpu", "cortex-a72", "-display", "none", "-serial", "stdio", "-kernel"]).arg(&bin),
        dir,
        &[],
        false,
        BOARD_LIMIT,
    );
    let got = String::from_utf8_lossy(&q.stdout).replace("\r\n", "\n");
    // a board still on is right only after a failed check, which freezes it
    if q.timed_out && !matches!(o, engines::Oracle::Stopped { .. }) {
        let so_far: String = got.chars().take(120).collect();
        return Outcome::Failed(format!("QEMU did not end (the board was not switched off); it printed {so_far:?}"));
    }
    let show = |s: &str| {
        let s = s.trim_end();
        if s.len() > 200 { format!("{:?}…", &s[..200]) } else { format!("{s:?}") }
    };
    match o {
        engines::Oracle::Ran { out } => {
            let want = String::from_utf8_lossy(&out).to_string();
            if got == want { Outcome::Same } else { Outcome::Differ(format!("oracle printed {}, QEMU {}", show(&want), show(&got))) }
        }
        engines::Oracle::Stopped { out, msg, .. } => {
            let want = String::from_utf8_lossy(&out).to_string();
            if got.starts_with(&want) { Outcome::Same } else { Outcome::Differ(format!("oracle stopped ({msg}) after {}, QEMU printed {}", show(&want), show(&got))) }
        }
        engines::Oracle::Timeout => Outcome::Failed("the oracle timed out".into()),
    }
}
