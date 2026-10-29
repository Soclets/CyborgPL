# A check that fails with no OS (check claude, 2026-10-26, check/corpus-bare):
# what was printed before it reaches the serial port, then the chip freezes on
# the trap (owner, 2026-10-08: "yea, freeze there").
fn dv [$n: i64] -> i64:
    ret 40 mod.floor ($n - 3);
;fn dv

MAIN:
    std::print("before", newln);
    for $i: i64 [from: 1, to: 5, by: 1]:
        std::print(dv($i), newln);
    ;for
    std::exit(code: 0);
;MAIN
