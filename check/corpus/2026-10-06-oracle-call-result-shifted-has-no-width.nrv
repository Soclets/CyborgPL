# Finding 2026-10-06 (check evaltest, seeds 55 and 123 with --avoid none, cut down by hand): `g[] shl 1`
# with g returning int8, in a comparison with a bare number. A shift's width comes from any operand
# whose type is known (owner: a, 2026-09-23), and a call's result has its return type, so 8 bits:
# 5 shl 1 is 10, and 3 > 10 is false. The oracle stops: "`shl` needs a width, and nothing here gives
# one". With the call's result held in a named int8 first it runs. CCA builds it (2 sites proven) and
# runs. Earlier runs of the same generator agreed (2026-10-05), so this is likely from c95ec8e.
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

fn g [] -> int8 {
    return 5;
}

MAIN {
    if 3 > (g[] shl 1) {
        put_u[1];
    } else {
        put_u[0];
    }
}
