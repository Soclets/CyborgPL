# Finding 2026-10-15 (check run, seed 183, cut down by hand): KAYO\n's layer 3 proves a narrowing
# that fails on a later trip of an outer loop, once the inner loop has run more than 16 times
# before that trip. Here to.u32['i'] gets 4, 1, then -2: the oracle prints 4 nine times, 1 nine
# times, then stops "-2 does not fit u32". KAYO\n claims the site proven (layer 3, solver), prints
# 4 nine more times for the third trip (not even -2 wrapped) and exits 0. Inner loop of 8 (2 x 8 = 16 iterations before the failing trip): checked.
# Of 9 (18): proven. Inner 5, failing on outer trip 4 (15): checked; on trip 5 (20): proven.
# Inner 3, failing on trip 6 (15): checked. Guess, unread: precog.md's 16 iterations per loop are
# counted across the outer loop's trips, and the trips past them are taken as unreachable, where
# the note has anything past the budget unproven.
%extern.C proc write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;

proc emit [$m: u64, $neg: bool]:
    var $buf: std::ls[u8] = [];
    if $neg:
        std::ls.append(lend.mut $buf, value: 45);
    ;if
    var $v: u64 = $m;
    var $ds: std::ls[u8] = [];
    loop:
        std::ls.append(lend.mut $ds, value: to.u8(48 + ($v mod.floor 10)));
        change $v to $v div.floor 10;
        if $v == 0:
            break;
        ;if
    ;loop
    for $k: u64 [from: std::ls.len(lend $ds), to: 1, by: -1]:
        std::ls.append(lend.mut $buf, value: $ds[pos: $k]);
    ;for
    std::ls.append(lend.mut $buf, value: 10);
    NO_FUCKING_CHECKS write(1, lend $buf, std::ls.len(lend $buf));
;proc emit

proc put_u [$n: u64]:
    emit($n, false);
;proc put_u

proc put_i [$n: i64]:
    if $n < 0:
        emit(to.u64(0 - ($n + 1)) + 1, true);
    else:
        emit(to.u64($n), false);
    ;if
;proc put_i
MAIN:
    for $i: i64 [from: 4, to: -2, by: -3]:
        for $j: u8 [from: 1, to: 9, by: 1]:
            put_u(to.u32($i));
        ;for
    ;for
;MAIN
