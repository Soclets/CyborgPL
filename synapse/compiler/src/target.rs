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
