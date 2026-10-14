# Finding 2026-09-30 (check run, seed 9148, cut down by hand): a wrong layer-3 proof. 'w10' counts down
# 5..0 in a loop, and 2030534677 shl 'w10' overflows uint32 for every count past 1. Precog claims the
# shift proven (shift_width, layer 3, solver) with its own operands {2030534677} and {0..5}; the binary
# prints wrapped values (552600224 = 64977109664 mod 2^32 first). The oracle stops at the first shift.
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

fn f1 ['p1': int64] -> int32 {
    let mut 'w10': int8 = 6;
    loop {
        if 'w10' equal_to 0 {
            break;
        }
        change 'w10' = 'w10' minus 1;
        put_u[to.uint32['p1' mod.floor 4294967296] shl to.uint32['w10']];
    }
    return 0;
}
MAIN {
    let 'x': int32 = f1[2030534677];
}
