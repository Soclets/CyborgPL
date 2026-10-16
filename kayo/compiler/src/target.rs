//! The target a build is for, chip-OS as `targets.txt` spells it
//! (`aarch64-macos`, `aarch64-none`). Set once a load knows it — the one
//! asked for, or the Neuron's default — before any file is parsed, since
//! `on.target` is decided while reading; the code generator and the linker
//! read it after. One per thread, as a load is.

use std::cell::RefCell;

thread_local! {
    static TARGET: RefCell<Option<String>> = const { RefCell::new(None) };
    static LAYOUT: RefCell<Option<(std::path::PathBuf, String)>> = const { RefCell::new(None) };
}

/// The linker script a no-OS Neuron brings (owner, 2026-10-09: "`c`
/// now"), found where its manifest is read: where it is, and its path as
/// the manifest writes it, for messages.
pub fn set_layout(p: Option<(std::path::PathBuf, String)>) {
    LAYOUT.with(|c| *c.borrow_mut() = p);
}

pub fn layout() -> Option<(std::path::PathBuf, String)> {
    LAYOUT.with(|c| c.borrow().clone())
}

/// This machine, chip-OS.
pub fn host() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

pub fn set(t: Option<&str>) {
    TARGET.with(|c| *c.borrow_mut() = t.map(str::to_string));
}

pub fn is_set() -> bool {
    TARGET.with(|c| c.borrow().is_some())
}

/// The build's target; this machine when none was named.
pub fn name() -> String {
    TARGET.with(|c| c.borrow().clone()).unwrap_or_else(host)
}

pub fn chip() -> String {
    let n = name();
    n.split_once('-').map(|(c, _)| c.to_string()).unwrap_or(n)
}

pub fn os() -> String {
    let n = name();
    n.split_once('-').map(|(_, o)| o.to_string()).unwrap_or_default()
}

/// No operating system under the program (owner, 2026-10-09:
/// `aarch64-none`): it is built on another machine and run on the bare
/// chip, so it is built from any host.
pub fn bare() -> bool {
    os() == "none"
}

/// LLVM's name for a no-OS target: `aarch64-unknown-none`.
pub fn bare_triple() -> String {
    format!("{}-unknown-none", chip())
}

/// C's machine-sized types (owner, 2026-10-10), as C's own names after
/// `C.`: what each is on this target.
pub const C_TYPES: [&str; 18] = [
    "char", "signed_char", "unsigned_char", "short", "unsigned_short", "int", "unsigned_int", "long", "unsigned_long", "long_long", "unsigned_long_long",
    "size_t", "ssize_t", "ptrdiff_t", "intptr_t", "uintptr_t", "wchar_t", "long_double",
];

/// What a C type is here: a whole number (signed, bits), a binary float
/// (bits), or why it is not built for this target yet.
pub enum CType {
    Int(bool, u32),
    Bin(u32),
    NotYet(String),
}

pub fn c_type(name: &str) -> Option<CType> {
    // Apple's arm64 ABI: `char` and `wchar_t` signed, `long double` a
    // double. The bare chip follows Arm's AAPCS64: `char` and `wchar_t`
    // unsigned, `long double` 128 bits.
    let apple = os() == "macos";
    Some(match name {
        "char" => CType::Int(apple, 8),
        "signed_char" => CType::Int(true, 8),
        "unsigned_char" => CType::Int(false, 8),
        "short" => CType::Int(true, 16),
        "unsigned_short" => CType::Int(false, 16),
        "int" => CType::Int(true, 32),
        "unsigned_int" => CType::Int(false, 32),
        "long" | "long_long" | "ssize_t" | "ptrdiff_t" | "intptr_t" => CType::Int(true, 64),
        "unsigned_long" | "unsigned_long_long" | "size_t" | "uintptr_t" => CType::Int(false, 64),
        "wchar_t" => CType::Int(apple, 32),
        "long_double" if apple => CType::Bin(64),
        "long_double" => CType::NotYet(format!("`C.long_double` is a 128-bit binary float on `{}`, and `bin128` is not built yet", crate::target::name())),
        _ => return None,
    })
}
