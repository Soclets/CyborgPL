# Finding 2026-10-06, ruled 2026-10-07 (owner: "b", oracle 0a653d62): a whole quotient that rounds to
# zero is 0, never -0 (div.X.ieee keeps IEEE's -0). 0.5 div.ceil (-1.75) on a brain16 is 0, text "0",
# 1 byte, as CCA gave; the oracle had kept -0 (seeds 25502, 26092).
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
    let 'a': brain16 = 0.5;
    let 'q': brain16 = 'a' div.ceil (-1.75);
    let 't' := to.useful.str.utf8['q'];
    put_u[str.bytes[lend 't']];
}
