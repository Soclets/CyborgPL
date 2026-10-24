# `on.target` chosen when compiling, `on.machine` asked when running (owner,
# 2026-10-06), mixed in one chain. Built for aarch64-macos: 10 from the
# whole target, 200 from the item branch, 3 because no Apple chip has SVE
# so the chain falls to the target branch, 30 from the chip: 243.
extern.C fn exit [$code: i32];
on.target.os windows {
    fn where [] -> i32 { return 1; }
} else {
    fn where [] -> i32 { return 2; }
}
MAIN {
    let mut $n: i32 = 0;
    on.target aarch64-macos {
        change $n = $n plus 10;
    } else on.target.chip x86_64 {
        let $k: i64 = 5;
        no_check inline.ASM { neg rax } with [$k -> rax, $k <- rax];
    } else {
        change $n = $n plus 20;
    }
    on.target.chip aarch64 { change $n = $n plus (100 times where()); }
    on.machine.chip sve { change $n = $n plus 1; } else on.target.chip aarch64 { change $n = $n plus 3; } else { change $n = $n plus 5; }
    on.machine.chip aarch64 {
        change $n = $n plus 30;
    } else {
        change $n = $n plus 40;
    }
    no_check my.exit($n mod.floor 256);
}
