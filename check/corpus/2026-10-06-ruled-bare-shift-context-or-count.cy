# Finding 2026-10-06, ruled 2026-10-07 (owner: "yea", oracle 4866c834): a shift's width comes from the
# shifted number, or its context when bare, never from the count. In to.i64[((-53) shr 'c') mod.floor
# 23] with 'c' an i8 of 9, the -53 is bare and a conversion is not a context, so nothing gives the
# shift a width: both engines refuse it ("`shr` needs a width"). Before the ruling the oracle ran it
# as a 64-bit shift (22) and CCA took the count's 8 bits (seed 4106).
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
    let 'c': i8 = 9;
    put_i[to.i64[((-53) shr 'c') mod.floor 23]];
}
