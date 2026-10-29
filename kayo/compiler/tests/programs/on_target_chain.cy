# `on.target` chosen when compiling, `on.machine` asked when running (owner,
# 2026-10-06), mixed in one chain. Built for aarch64-macos: 10 from the
# whole target, 200 from the item branch, 3 because no Apple chip has SVE
# so the chain falls to the target branch, 30 from the chip: 243.
%extern.C proc exit [let $code: i32];
on.target.os windows:
    fn where [] -> i32:
        ret 1;
    ;fn where
else:
    fn where [] -> i32:
        ret 2;
    ;fn where
;on.target.os
MAIN:
    var $n: i32 = 0;
    on.target aarch64-macos:
        change $n to $n + 10;
    else on.target.chip x86_64:
        let $k: i64 := 5;
        no_check inline.ASM with [$k -> rax, $k <- rax]:
            neg rax
        ;inline.ASM
    else:
        change $n to $n + 20;
    ;on.target
    on.target.chip aarch64:
        change $n to $n + (100 x where());
    ;on.target.chip
    on.machine.chip sve:
        change $n to $n + 1;
    else on.target.chip aarch64:
        change $n to $n + 3;
    else:
        change $n to $n + 5;
    ;on.machine.chip
    on.machine.chip aarch64:
        change $n to $n + 30;
    else:
        change $n to $n + 40;
    ;on.machine.chip
    no_check my.exit($n mod.floor 256);
;MAIN
