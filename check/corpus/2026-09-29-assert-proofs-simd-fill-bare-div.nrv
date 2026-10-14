# Finding 2026-09-29 (check run, seed 5120, shrunk then cut by hand): simd.fill[15059671820752541943 div 3]
# into simd[uint64, 2] is 5019890606917513981 (bare numbers alone are exact). The normal build and the
# oracle agree; the --assert-proofs build stops (proof failed: the value can be outside what a uint64
# holds), as if the literal past int64 were divided as a signed int64. A plain fill of the literal,
# or of a uint64 name holding the quotient, is fine. The same family as assert-proofs-match-arm-bare-div.
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
    let 's': simd[uint64, 2] = simd.fill[value 15059671820752541943 div.floor 3];
    put_u[to.uint64[simd.max['s']]];
}
