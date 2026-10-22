# Finding 2026-09-29 (check run, seed 1027, shrunk): put_u[0 - 1] in a loop fails at the call (the
# argument -1 does not fit u64), and CCA rightly notes it as a check that may never run. But it
# also carries that -1 into put_u -> emit as if it had arrived, and with the real call put_u[0] after
# the loop it refuses emit's 'v' div 10 as failing "every time it reaches here" (line 15). From
# put_u[0] it is 0 div 10. The oracle stops at line 47, the call, as it should.
# check claude generator, seed 1027
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
    no_check write[1, lend 'buf', std::list.length[lend 'buf']];
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
    let 'w3': i8 = 6;
    while 'w3' > 0 {
        change 'w3' = 'w3' minus 1;
        for 'i5': i64 [from: 0, to: 0, by: 1] {
            put_u[0 minus 1];
        }
    }
    put_u[0];
}
