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
    // itself by it, `CyDT 5eac982a KAYO\n`, and by `5eac982a+` when what it
    // is built from has changes not committed (owner, 2026-10-16: "Yeah,
    // add the +"). Run again whenever the checkout moves or one of those
    // files changes.
    let here = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let built_from: Vec<std::path::PathBuf> = ["src", "build.rs", "Cargo.toml", "lld", "../base", "../limbs", "../Cargo.toml", "../Cargo.lock", "../targets.txt", "../../spellings.cyr"].iter().map(|p| here.join(p)).collect();
    let git = |args: &[&str]| {
        std::process::Command::new("git").args(args).current_dir(&here).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let commit = match git(&["rev-parse", "HEAD"]) {
        Some(h) => {
            let mut status = vec!["status", "--porcelain", "--"];
            let paths: Vec<String> = built_from.iter().map(|p| p.display().to_string()).collect();
            status.extend(paths.iter().map(String::as_str));
            let changed = git(&status).is_some_and(|out| !out.is_empty());
            format!("{}{}", h.chars().take(8).collect::<String>(), if changed { "+" } else { "" })
        }
        None => "unknown".into(),
    };
    println!("cargo:rustc-env=KAYO_COMMIT={}", commit);
    if let Some(dir) = git(&["rev-parse", "--absolute-git-dir"]) {
        let mut watch = vec![format!("{}/HEAD", dir), format!("{}/packed-refs", dir), format!("{}/index", dir)];
        if let Some(r) = git(&["symbolic-ref", "-q", "HEAD"]) {
            watch.push(format!("{}/{}", dir, r));
        }
        watch.extend(built_from.iter().map(|p| p.display().to_string()));
        for w in watch.iter().filter(|w| std::path::Path::new(w).exists()) {
            println!("cargo:rerun-if-changed={}", w);
        }
    }
    // `incl.C` asks the Clang beside this LLVM to read C headers.
    println!("cargo:rustc-env=KAYO_LLVM_PREFIX={}", llvm);
    // The shim is compiled again only when it or the libraries it is
    // compiled against change: the script runs on every edit of the
    // compiler now, for the `+` above.
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let key = format!("{}\n{}\n{}", lld, llvm, std::fs::read_to_string(here.join("lld/shim.cpp")).unwrap_or_default());
    let key_at = out.join("cca_lld.key");
    if out.join("libcca_lld.a").is_file() && std::fs::read_to_string(&key_at).ok().as_deref() == Some(key.as_str()) {
        println!("cargo:rustc-link-search=native={}", out.display());
        println!("cargo:rustc-link-lib=static=cca_lld");
    } else {
        compile_shim(&lld, &llvm);
        let _ = std::fs::write(&key_at, &key);
    }
    println!("cargo:rustc-link-search=native={}/lib", lld);
    for lib in ["lldMachO", "lldELF", "lldCOFF", "lldWasm", "lldCommon"] {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }
    // lld's libraries find each other through the rpath.
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", lld);
    println!("cargo:rustc-link-lib=dylib=c++");
}

fn compile_shim(lld: &str, llvm: &str) {
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
}
