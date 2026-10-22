# Finding 2026-10-08 (check run --risk 20, seed 36150, cut down by hand): (6729433359276983393 times
# 17924805223061868988) plus 'v', 'v' a u64. The bare product is exact on its own, but plus with a
# typed operand is plain math with the 64-bit limit in between (design/operators.md): the sum,
# 120623782226614850658282569013217716289, is past it, a check site. The oracle stops there. The CCA
# binary runs on (likely since f0cbd125 worked a bare group beside a typed operand at its own size).
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

fn b1 [@y, @x] [lent @y 'a': std::list[u16], lent @y 'b': std::list[u16], lent @x 'c': std::list[u16]] -> lent @x std::list[u16] {
    return lend 'c';
}

fn b2 [@x, @y] [lent 'a': std::list[u16], lent @x 'b': std::list[u16], lent @y 'c': std::list[u16]] -> lent @x std::list[u16] {
    return b1[lend 'b', lend 'a', lend 'b'];
}

fn b3 [@y, @x] [lent @y 'a': std::list[u16], lent @x 'b': std::list[u16], lent @x 'c': std::list[u16]] -> lent @x std::list[u16] {
    if std::list.length[lend 'c'] at_least std::list.length[lend 'a'] {
        return b2[lend 'a', lend 'c', lend 'c'];
    }
    return b1[lend 'c', lend 'c', lend 'b'];
}

fn b4 [@y, @x] [lent @y 'a': std::list[u16], lent @y 'b': std::list[u16], lent @x 'c': std::list[u16]] -> lent @x std::list[u16] {
    return lend 'c';
}

MAIN {
    let 'v': u64 = 5;
    let 'r': u64 = ((6729433359276983393 times 17924805223061868988) plus 'v') mod.floor 7;
    put_u['r'];
}
