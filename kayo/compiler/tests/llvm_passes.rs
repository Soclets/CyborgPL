//! kayo's own LLVM passes in C++ (`kayo/llvm-passes/`, owner, 2026-10-17),
//! each tested on hand-written IR with AddressSanitizer and
//! UndefinedBehaviorSanitizer, built with the Clang beside the LLVM kayo
//! links.

use std::path::Path;
use std::process::Command;

#[test]
fn recursive_inlining_under_the_sanitizers() {
    let llvm = env!("KAYO_LLVM_PREFIX");
    let passes = Path::new(env!("CARGO_MANIFEST_DIR")).join("../llvm-passes");
    let exe = std::env::temp_dir().join(format!("kayo-recursive-inline-test-{}", std::process::id()));
    let built = Command::new(format!("{}/bin/clang++", llvm))
        .args(["-std=c++20", "-fno-rtti", "-fno-exceptions", "-Wall", "-Wextra", "-Werror"])
        .arg(format!("-isystem{}/include", llvm))
        .args(["-D__STDC_CONSTANT_MACROS", "-D__STDC_FORMAT_MACROS", "-D__STDC_LIMIT_MACROS"])
        .args(["-fsanitize=address,undefined", "-fno-sanitize-recover=all", "-g"])
        .arg(passes.join("test/recursive_inline_test.cpp"))
        .arg(passes.join("recursive_inline.cpp"))
        .arg(format!("-L{}/lib", llvm))
        .arg("-lLLVM")
        .arg(format!("-Wl,-rpath,{}/lib", llvm))
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(built.status.success(), "the test did not build:\n{}", String::from_utf8_lossy(&built.stderr));
    let ran = Command::new(&exe).output().unwrap();
    let _ = std::fs::remove_file(&exe);
    assert!(ran.status.success() && String::from_utf8_lossy(&ran.stdout).contains("ok"), "the recursive inliner failed its test:\n{}{}", String::from_utf8_lossy(&ran.stdout), String::from_utf8_lossy(&ran.stderr));
}
