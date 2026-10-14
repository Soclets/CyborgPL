# Finding 2026-10-09 (check run, seed 44056, cut down by hand): a function returning uint8 has
# `return true;` on a path that never runs. A return's type is the function's (a rule about the code,
# like a let over a visible name, caba9b84): Synapse refuses it ("expected a value of the return type
# (uint8), found a `bool`"). The oracle runs the program (exit 0); on a path that runs it refuses too.
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

fn b1 [@y, @x] [lent @y 'a': list[uint16], lent @x 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    if list.length[lend 'b'] at_least list.length[lend 'c'] {
        return lend 'c';
    }
    return lend 'b';
}

fn b2 [@x] [lent @x 'a': list[uint16]] -> lent @x list[uint16] {
    return lend 'a';
}

fn b3 [@x, @y] [lent @x 'a': list[uint16], lent @y 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    if list.length[lend 'c'] at_least list.length[lend 'a'] {
        return b2[lend 'a'];
    }
    return lend 'c';
}

fn b4 [@x] [lent 'a': list[uint16], lent 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    return lend 'c';
}

fn g ['p': uint8] -> uint8 {
    if false { return true; }
    return 1;
}

MAIN {
    put_u[to.uint64[g[1]]];
}
