# Finding 2026-10-07 (check run --borrow-wrong 1000, seed 28563, shrunk): if 0 >/== (-5988905564081113680)
# times (-4). Bare numbers alone are worked out exactly and a comparison compares true values: the
# product is 23955622256324454720, so the test is false and the oracle prints 1 (the else). The CCA
# binary prints 0: it takes the then branch, as if the product had wrapped.
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

fn b1 [@x] [lent @x 'a': list[int16], lent @x 'b': list[int16]] -> lent @x list[int16] {
    if list.length[lend 'b'] > 2 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@x] [lent @x 'a': list[int16]] -> lent @x list[int16] {
    if list.length[lend 'a'] > 2 {
        return b1[lend 'a', lend 'a'];
    }
    return lend 'a';
}

fn b3 [@x] [lent @x 'a': list[int16], lent @x 'b': list[int16]] -> lent @x list[int16] {
    return lend 'a';
}

MAIN {
    if 0 at_least (-5988905564081113680) times (-4) {
        put_u[0];
    } else {
        put_u[1];
    }
}
