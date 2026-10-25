//! The kernel, compiled by Apple's compiler as kayo compiles a block.
use std::process::Command;

fn main() {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let air = out.join("kernel.air");
    let lib = out.join("kernel.metallib");
    println!("cargo:rerun-if-changed=kernel.metal");
    let ok = Command::new("xcrun").args(["-sdk", "macosx", "metal", "-c", "kernel.metal", "-o"]).arg(&air).status().map(|s| s.success()).unwrap_or(false)
        && Command::new("xcrun").args(["-sdk", "macosx", "metallib"]).arg(&air).arg("-o").arg(&lib).status().map(|s| s.success()).unwrap_or(false);
    assert!(ok, "`xcrun metal` could not compile kernel.metal");
}
