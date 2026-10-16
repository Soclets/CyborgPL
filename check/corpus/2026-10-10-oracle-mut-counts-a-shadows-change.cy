# Finding 2026-10-10 (check run, 8 seeds such as 48033, cut down by hand): `let mut 'x'`, then `shadow
# 'x'` in the same scope, then `change 'x'`, which changes the shadow. Nothing changes the first 'x',
# so its `mut` is a promise broken (owner, 2026-10-10): Synapse refuses ("'x' is `mut`, but nothing
# changes it"). The oracle runs it (prints 3): it counts the shadow's change for the shadowed name.
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
    let mut 'x': int32 = 1;
    shadow 'x': int32 = 2;
    change 'x' = 3;
    put_i[to.int64['x']];
}
