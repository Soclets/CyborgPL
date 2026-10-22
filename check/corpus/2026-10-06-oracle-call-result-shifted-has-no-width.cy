# Finding 2026-10-06 (check evaltest, seeds 55 and 123 with --avoid none, cut down by hand): `g[] shl 1`
# with g returning i8, in a comparison with a bare number. A shift's width comes from any operand
# whose type is known (owner: a, 2026-09-23), and a call's result has its return type, so 8 bits:
# 5 shl 1 is 10, and 3 > 10 is false. The oracle stops: "`shl` needs a width, and nothing here gives
# one". With the call's result held in a named i8 first it runs. CCA builds it (2 sites proven) and
# runs. Earlier runs of the same generator agreed (2026-10-05), so this is likely from c95ec8e.
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

fn g [] -> i8 {
    return 5;
}

MAIN {
    if 3 > (g[] shl 1) {
        put_u[1];
    } else {
        put_u[0];
    }
}
