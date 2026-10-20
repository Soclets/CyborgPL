# Finding 2026-10-05 (check run, seeds 20003, 20067 and others, cut down by hand): an exact answer
# under a unary minus, shifted: (-('a' minus 1)) shr 1 with an i8 'a', into an i8. Its only typed
# operand is i8, so it shifts at 8 bits (owner, 2026-10-03; 363f19f) and the oracle prints -10.
# CCA refuses: "these have different sizes (or none left to see)". Without the minus, or with
# (-'a') shr 1, both agree; the same with times.wrap, xor, or.bit beside a negated exact answer.
# check claude generator, seed 20067
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

fn emit ['m': u64, 'neg': bool] {
    let mut 'buf': list[u8] = [];
    if 'neg' { list.append[lend.mut 'buf', value: 45]; }
    let mut 'v': u64 = 'm';
    let mut 'ds': list[u8] = [];
    loop {
        list.append[lend.mut 'ds', value: to.u8[48 plus ('v' mod.floor 10)]];
        change 'v' = 'v' div.floor 10;
        if 'v' equal_to 0 { break; }
    }
    for 'k': u64 [from: list.length[lend 'ds'], to: 1, by: -1] {
        list.append[lend.mut 'buf', value: 'ds'[pos: 'k']];
    }
    list.append[lend.mut 'buf', value: 10];
    NO_FUCKING_CHECKS write[1, lend 'buf', list.length[lend 'buf']];
}

fn put_u ['n': u64] {
    emit['n', false];
}

fn put_i ['n': i64] {
    if 'n' < 0 {
        emit[to.u64[0 minus ('n' plus 1)] plus 1, true];
    } else {
        emit[to.u64['n'], false];
    }
}

fn b1 [@x] [lent @x 'a': list[u16], lent @x 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    if list.length[lend 'b'] > 1 {
        return lend 'b';
    }
    return lend 'a';
}

fn b2 [@y, @x] [lent @y 'a': list[u16], lent @y 'b': list[u16], lent @x 'c': list[u16]] -> lent @x list[u16] {
    return lend 'c';
}

fn m1 [lent.mut 'a': list[u16], lent 'b': list[u16]] {
    if list.length[lend 'b'] > 0 {
        list.append[lend.mut 'a', value: 1];
    }
}
MAIN {
    let 'a': i8 = 20;
    let 'b': i8 = (-('a' minus 1)) shr 1;
    put_i[to.i64['b']];
}
