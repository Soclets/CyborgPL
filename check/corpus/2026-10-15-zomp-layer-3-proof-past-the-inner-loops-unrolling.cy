# Finding 2026-10-15 (check run, seed 183, cut down by hand): Zomp's layer 3 proves a narrowing
# that fails on a later trip of an outer loop, once the inner loop has run more than 16 times
# before that trip. Here to.u32['i'] gets 4, 1, then -2: the oracle prints 4 nine times, 1 nine
# times, then stops "-2 does not fit u32". Zomp claims the site proven (layer 3, solver), prints
# 4 nine more times for the third trip (not even -2 wrapped) and exits 0. Inner loop of 8 (2 x 8 = 16 iterations before the failing trip): checked.
# Of 9 (18): proven. Inner 5, failing on outer trip 4 (15): checked; on trip 5 (20): proven.
# Inner 3, failing on trip 6 (15): checked. Guess, unread: precog.md's 16 iterations per loop are
# counted across the outer loop's trips, and the trips past them are taken as unreachable, where
# the note has anything past the budget unproven.
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append(lend.mut 'buf', value: 45); }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    loop {
        std::list.append(lend.mut 'ds', value: to.u8(48 plus ('v' mod.floor 10)));
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': u64 [from: std::list.length(lend 'ds'), to: 1, by: -1] {
        std::list.append(lend.mut 'buf', value: 'ds'[pos: 'k']);
    }
    std::list.append(lend.mut 'buf', value: 10);
    NO_FUCKING_CHECKS write(1, lend 'buf', std::list.length(lend 'buf'));
}

fn put_u ['n': u64] {
    emit('n', false);
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit(to.u64(0 minus ('n' plus 1)) plus 1, true);
    } else {
        emit(to.u64('n'), false);
    }
}
MAIN { for 'i': i64 [from: 4, to: -2, by: -3] { for 'j': u8 [from: 1, to: 9, by: 1] { put_u(to.u32('i')); } } }
