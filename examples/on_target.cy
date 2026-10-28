# Code for one target (owner, 2026-10-06): `on.target` is chosen when compiling, and a branch it rules
# out is not compiled at all; `on.machine` asks the actual machine once, when running.
# The oracle's target is aarch64-macos.

on.target.os windows:
    fn where [] -> str.utf8:
        ret "windows";
    ;fn where
else:
    fn where [] -> str.utf8:
        ret "not windows";
    ;fn where
;on.target.os

MAIN:
    on.target aarch64-macos:
        std::println("built for aarch64-macos");
    else on.target.chip x86_64:
        let $n: i64 := 5;
        no_check inline.ASM with [$n -> rax, $n <- rax]:   # never compiled here: x86 assembly
            neg rax
        ;inline.ASM
    else:
        std::println("built for something else");
    ;on.target
    on.target.chip aarch64:
        std::println("an ARM chip, ", where());
    ;on.target.chip
    on.machine.chip aarch64:
        std::println("running on ARM");
    else:
        std::println("running elsewhere");
    ;on.machine.chip
;MAIN
