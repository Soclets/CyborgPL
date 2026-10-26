# Finding 2026-10-26 (check bare, seeds 39 and 71): on a chip with no operating system (aarch64-none, QEMU's virt
# board; `check bare` adds the board module of examples/bare_station and a manifest naming it as the target's
# `output` and `exit`), a check that fails at run time. The oracle prints "before", "0", "0", then stops at the
# `mod` by zero (line 7). kayo's build prints nothing at all, not even what came before, and never switches the
# board off: its grouped prints are not sent and the target's `exit` is not reached.
fn dv [$n: i64] -> i64 {
    return 40 mod.floor ($n - 3);
}

MAIN {
    std::print.useful("before", newln);
    for $i: i64 [from: 1, to: 5, by: 1] {
        std::print.useful(dv($i), newln);
    }
    std::exit(code: 0);
}
