# Finding 2026-10-08 (check run --risk 20, seed 36150, cut down by hand): (6729433359276983393 times
# 17924805223061868988) plus 'v', 'v' a uint64. The bare product is exact on its own, but plus with a
# typed operand is plain math with the 64-bit limit in between (design/operators.md): the sum,
# 120623782226614850658282569013217716289, is past it, a check site. The oracle stops there. The CCA
# binary runs on (likely since f0cbd125 worked a bare group beside a typed operand at its own size).
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

fn b1 [@y, @x] [lent @y 'a': list[uint16], lent @y 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    return lend 'c';
}

fn b2 [@x, @y] [lent 'a': list[uint16], lent @x 'b': list[uint16], lent @y 'c': list[uint16]] -> lent @x list[uint16] {
    return b1[lend 'b', lend 'a', lend 'b'];
}

fn b3 [@y, @x] [lent @y 'a': list[uint16], lent @x 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    if list.length[lend 'c'] at_least list.length[lend 'a'] {
        return b2[lend 'a', lend 'c', lend 'c'];
    }
    return b1[lend 'c', lend 'c', lend 'b'];
}

fn b4 [@y, @x] [lent @y 'a': list[uint16], lent @y 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    return lend 'c';
}

MAIN {
    let 'v': uint64 = 5;
    let 'r': uint64 = ((6729433359276983393 times 17924805223061868988) plus 'v') mod.floor 7;
    put_u['r'];
}
