# Finding 2026-10-02 (check run, seed 10411, cut down by hand): inside for 'x' in lend.mut 'xs',
# change 'x' = 'v' copies a u8; CCA then treats 'x' as borrowing 'v' and refuses a later read of
# 'v' ("'v' cannot be used here: 'x' still borrows it lent.mut"). The oracle prints 5 7 5 7 7.
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': std::list[u8] = [];
    if 'neg' { std::list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': std::list[u8] = [];
    std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        std::list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = std::list.length[lend 'ds'];
    while 'k' at_least 1 {
        std::list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' minus 1;
    }
    std::list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', std::list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' at_least 0 {
        emit[to.u64['n'], false];
    } else {
        let 'up': i64 = -('n' plus 1);
        emit[to.u64['up'] plus 1, true];
    }
}

MAIN {
    let 'v': u8 = 5;
    let mut 'xs': std::list[u8] = [1, 2];
    for 'x' in lend.mut 'xs' {
        change 'x' = 'v';
        change 'x' = 7;
        put_u[to.u64['v']];
        put_u[to.u64['x']];
    }
    put_u[to.u64['xs'[pos: 1]]];
}
