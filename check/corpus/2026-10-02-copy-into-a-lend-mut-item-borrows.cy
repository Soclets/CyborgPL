# Finding 2026-10-02 (check run, seed 10411, cut down by hand): inside for 'x' in lend.mut 'xs',
# change 'x' = 'v' copies a uint8; CCA then treats 'x' as borrowing 'v' and refuses a later read of
# 'v' ("'v' cannot be used here: 'x' still borrows it lent.mut"). The oracle prints 5 7 5 7 7.
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

MAIN {
    let 'v': uint8 = 5;
    let mut 'xs': list[uint8] = [1, 2];
    for 'x' in lend.mut 'xs' {
        change 'x' = 'v';
        change 'x' = 7;
        put_u[to.uint64['v']];
        put_u[to.uint64['x']];
    }
    put_u[to.uint64['xs'[pos 1]]];
}
