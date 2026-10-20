# Code for one target (owner, 2026-10-06): `on.target` is chosen when compiling, and a branch it rules
# out is not compiled at all; `on.machine` asks the actual machine once, when running.
# The oracle's target is aarch64-macos.

on.target.os windows {
    fn where [] -> str.utf8 { return "windows"; }
} else {
    fn where [] -> str.utf8 { return "not windows"; }
}

MAIN {
    on.target aarch64-macos {
        std::print.useful["built for aarch64-macos", newln];
    } else on.target.chip x86_64 {
        let 'n': i64 = 5;
        no_check inline.ASM { neg rax } with ['n' -> rax, 'n' <- rax];   # never compiled here: x86 assembly
    } else {
        std::print.useful["built for something else", newln];
    }
    on.target.chip aarch64 { std::print.useful["an ARM chip, ", where[], newln]; }
    on.machine.chip aarch64 {
        std::print.useful["running on ARM", newln];
    } else {
        std::print.useful["running elsewhere", newln];
    }
}
