# Finding 2026-10-07 (check run --risk 20, seed 28093, cut down by hand): a list[u64] literal whose
# item is ((16874889984015390149 pow 8) mod.floor 665) plus 333. Bare numbers alone are worked out
# exactly: 454, which the oracle prints. CCA refuses: "`pow` goes past 64 bits in between" (its
# answer shown as 2^127 - 1). Stored by a plain let 'r': u64 = … it builds (6232bb14).
# check claude generator, seed 1
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': list[u8] = [];
    list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
    change 'v' = 'v' div.floor 10;
    while 'v' > 0 {
        list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
    }
    let mut 'k': u64 = list.length[lend 'ds'];
    while 'k' at_least 1 {
        list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
        change 'k' = 'k' minus 1;
    }
    list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
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

fn b1 [@x] [lent @x 'a': list[i16], lent @x 'b': list[i16]] -> lent @x list[i16] {
    if list.length[lend 'b'] > 2 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@x] [lent @x 'a': list[i16]] -> lent @x list[i16] {
    if list.length[lend 'a'] > 2 {
        return b1[lend 'a', lend 'a'];
    }
    return lend 'a';
}

fn b3 [@x] [lent @x 'a': list[i16], lent @x 'b': list[i16]] -> lent @x list[i16] {
    return lend 'a';
}

MAIN {
    let 'xs': list[u64] = [((16874889984015390149 pow 8) mod.floor 665) plus 333,];
    put_u['xs'[pos: 1]];
}
