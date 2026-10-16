# Finding 2026-10-16 (a width probe, by hand, after 59c45ac6): two more places where a shift of a bare
# number has no width (a conversion is not a context, 2026-10-07 corpus; print gives none) that the
# oracle refuses only where they run. In a branch never taken, kayo refuses `to.int64[5 shr
# to.uint8[1]]` when building; the oracle prints 7. Taken, the oracle refuses it ("`shr` needs a
# width"), and the same for std::print.useful[5 shr to.uint8[1], newln].
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
MAIN {
    let 'p': bool = to.int64[1] equal_to to.int64[2];
    if 'p' {
        let 'r' := to.int64[5 shr to.uint8[1]];
        std::print.useful[5 shr to.uint8[1], newln];
    }
    put_u[7];
}
