# Finding 2026-10-17 (check run, seeds 2050 and 2058, cut down by hand): Precog's layer-2 value set for
# a mod's divisor in mutual recursion keeps four values and drops the rest, then proves the divisor is
# never zero. ev[10, 50] reaches od with 'n' 9, 7, 5, 3, 1, so ('acc' x 3) mod.floor ('n' - 1) is a mod
# by zero at the last: the oracle stops there. Precog claims the divisor's values are {2, 4, 6, 8} and
# proves "the divisor is never zero"; the check is gone and kayo prints 60. From ev[8] the set is
# {0, 2, 4, 6}, the site is checked and kayo stops too. Not the recursion inlining (0ed08993), which
# leaves mutual recursion alone.
# check claude generator (printer only)
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': uint64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value 'ds'[pos 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': uint64] {
    emit['n', false];
}

fn put_i ['n': int64] {
    if 'n' at_least 0 {
        emit[to.uint64['n'], false];
    } else {
        let 'up': int64 = -('n' plus 1);
        emit[to.uint64['up'] plus 1, true];
    }
}
fn ev ['n': uint32[min 0, max 23], 'acc': int64] -> int64 {
    if 'n' equal_to 0 { return 'acc'; }
    put_i['acc'];
    return od['n' - 1, ('acc' + 20) mod.floor 3710];
}
fn od ['n': uint32[min 0, max 23], 'acc': int64] -> int64 {
    if 'n' equal_to 0 { return 'acc'; }
    return ev['n' - 1, ('acc' x 3) mod.floor ('n' - 1)];
}
MAIN {
    put_i[ev[10, 50]];
}
