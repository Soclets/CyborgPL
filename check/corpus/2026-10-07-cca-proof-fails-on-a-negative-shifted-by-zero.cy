# Finding 2026-10-07 (check run --risk 20, seed 28123, shrunk): ((0 minus 'v30') shr 0) mod.floor 1
# with 'v30' a u32 of 1, into a u32. The oracle prints 0. CCA builds it, but its --assert-proofs
# binary fails: "the shift count can be outside 0 to the width - 1", though the count is 0. The value
# shifted is -1 at the u32 width; with ('v30' plus 1) instead both agree. Ruled since (owner: a,
# 2026-10-07; oracle 3fd09056): a shifted number must fit its width first, so -1 there is a check
# site, and the oracle now stops on it ("-1 does not fit u32").
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
    let 'v30': u32 = 1;
    let 'r': u32 = ((0 minus 'v30') shr 0) mod.floor 1;
    put_u[to.u64['r']];
}
