//! lld, built into CCA through its library (owner, 2026-10-08: "`a`"):
//! the shim in `lld/shim.cpp`, compiled with the C++ compiler, and lld's
//! libraries, Homebrew's lld 22 for now (`KAYO_LLD_PREFIX`, set in
//! `.cargo/config.toml`), beside the shared LLVM inkwell links.

fn main() {
    let lld = std::env::var("KAYO_LLD_PREFIX").unwrap_or_else(|_| "/opt/homebrew/opt/lld@22".into());
    let llvm = std::env::var("LLVM_SYS_221_PREFIX").unwrap_or_else(|_| "/opt/homebrew/opt/llvm@22".into());
    println!("cargo:rerun-if-changed=lld/shim.cpp");
    println!("cargo:rerun-if-env-changed=KAYO_LLD_PREFIX");
    // The commit this KAYO\n is built from: a build between releases names
    // itself by it, `CyDT 5eac982a KAYO\n` (owner, 2026-10-16). Built again
    // whenever the checkout moves to another commit.
    let git = |args: &[&str]| {
        std::process::Command::new("git").args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let commit = git(&["rev-parse", "HEAD"]).map(|h| h.chars().take(8).collect::<String>()).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=KAYO_COMMIT={}", commit);
    if let Some(dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        let mut watch = vec![format!("{}/HEAD", dir), format!("{}/packed-refs", dir)];
        if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"]) {
            watch.push(format!("{}/{}", dir, r));
        }
        for w in watch.iter().filter(|w| std::path::Path::new(w).exists()) {
            println!("cargo:rerun-if-changed={}", w);
        }
    }
    // `incl.C` asks the Clang beside this LLVM to read C headers.
    println!("cargo:rustc-env=KAYO_LLVM_PREFIX={}", llvm);
    cc::Build::new()
        .cpp(true)
        .file("lld/shim.cpp")
        .include(format!("{}/include", lld))
        .include(format!("{}/include", llvm))
        .flag("-std=c++17")
        // LLVM is built without RTTI or exceptions.
        .flag("-fno-rtti")
        .flag("-fno-exceptions")
        .warnings(false)
        .compile("cca_lld");
    println!("cargo:rustc-link-search=native={}/lib", lld);
    for lib in ["lldMachO", "lldELF", "lldCOFF", "lldWasm", "lldCommon"] {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }
    // lld's libraries find each other through the rpath.
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", lld);
    println!("cargo:rustc-link-lib=dylib=c++");
}
