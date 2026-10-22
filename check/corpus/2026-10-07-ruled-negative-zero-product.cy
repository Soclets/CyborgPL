# Finding 2026-10-07, ruled the same day (owner: "a", oracle 72d6946d): no bin or brain16 result is
# ever -0; plus/minus/times/div.X.ieee and neg.ieee keep IEEE's -0. A brain16 zero times (-0.3) is 0,
# text "0", 1 byte, as CCA gave; the oracle had kept -0 (seed 30208).
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

fn b1 [@x] [lent @x 'a': std::list[i16], lent @x 'b': std::list[i16]] -> lent @x std::list[i16] {
    if std::list.length[lend 'b'] > 2 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@x] [lent @x 'a': std::list[i16]] -> lent @x std::list[i16] {
    if std::list.length[lend 'a'] > 2 {
        return b1[lend 'a', lend 'a'];
    }
    return lend 'a';
}

fn b3 [@x] [lent @x 'a': std::list[i16], lent @x 'b': std::list[i16]] -> lent @x std::list[i16] {
    return lend 'a';
}

MAIN {
    let 'w4': brain16 = 0.1;
    let 'w6': brain16 = 'w4' minus 'w4';
    let 'w7': brain16 = 'w6' times (-0.3);
    let 't15' := to.useful.str.utf8['w7'];
    put_u[std::str.bytes[lend 't15']];
}
