# Finding 2026-10-05 (check run, 85 seeds such as 20020, cut down by hand): a bare number shifted
# by an exact answer, 7 shr (('a' minus 1) mod.floor 32), with a uint32 'a', stored into a uint32.
# A shift's width comes from any operand whose type is known, a literal taking its context's
# (owner: a, 2026-09-23), so 32 bits; the oracle prints 0. CCA refuses: "`shr` only knows a
# declared width". The same inside a comparison with a bare number (seed 20020).
# check claude generator, seed 20067
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;

fn emit ['m': uint64, 'neg': bool] {
    let mut 'buf': list[uint8] = [];
    if 'neg' { list.append[lend.mut 'buf', value 45]; }
    let mut 'v': uint64 = 'm';
    let mut 'ds': list[uint8] = [];
    loop {
        list.append[lend.mut 'ds', value to.uint8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': uint64 from list.length[lend 'ds'] to 1 by -1 {
        list.append[lend.mut 'buf', value 'ds'[pos 'k']];
    }
    list.append[lend.mut 'buf', value 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': uint64] {
    emit['n', false];
}

fn put_i ['n': int64] {
    if 'n' < 0 {
        emit[to.uint64[0 minus ('n' plus 1)] plus 1, true];
    } else {
        emit[to.uint64['n'], false];
    }
}

fn b1 [@x] [lent @x 'a': list[uint16], lent @x 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    if list.length[lend 'b'] > 1 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@y, @x] [lent @y 'a': list[uint16], lent @y 'b': list[uint16], lent @x 'c': list[uint16]] -> lent @x list[uint16] {
    return lend 'c';
}

fn m1 [lent.mut 'a': list[uint16], lent 'b': list[uint16]] {
    if list.length[lend 'b'] > 0 {
        list.append[lend.mut 'a', value 1];
    }
}
MAIN {
    let 'a': uint32 = 5;
    let 'b': uint32 = 7 shr (('a' minus 1) mod.floor 32);
    put_u[to.uint64['b']];
}
