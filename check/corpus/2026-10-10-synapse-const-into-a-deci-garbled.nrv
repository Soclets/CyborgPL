# Finding 2026-10-10 (check run, seed 48186, cut down by hand): a body-level const (owner, 2026-10-10)
# converted to a deci128. const 'c': int64 = 1234, then to.deci128['c']: the oracle has 1234, and
# to.int64['d' times 1] gives 1234. Synapse's deci comes out wrong (printed stored, 1234 shows as
# 0.000…0001234 with a huge negative exponent; -9223372036854775807 shows as -0), so it gives -1. The
# same value from a `let` converts right.
# check claude generator, seed 1
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

struct Counter {
    now: int32,
    stop: int32
}

fn Counter.next [lent.mut 'c': Counter] -> optional[int32] {
    if 'c'.now at_least 'c'.stop { return optional.none; }
    change 'c'.now = 'c'.now plus 1;
    return optional.value['c'.now];
}

fn half ['n': int32] -> optional[int32] {
    if ('n' mod.floor 2) equal_to 0 { return optional.value['n' div.floor 2]; }
    return optional.none;
}

MAIN {
    const 'c': int64 = 1234;
    let 'd' := to.deci128['c'];
    let 'z' := to.int64['d' times 1] otherwise -1;
    put_i['z'];
}
