# Finding 2026-10-15 (check run, seed 202, cut down by hand): Synapse's layer 3 proves a division
# by zero can't happen when the same function earlier printed the exact quotient
# (-9223372036854775808) div.floor (-1), which is 2^63, narrowed into uint64. f[f[22, 19], -900]
# divides -900 by 0 on the second call: the oracle stops "division by zero" at that line. Synapse's
# claim for that site is "proven, layer 3, solver" (division_by_zero, past_64_bits and the int64
# narrowing all at once), its check is gone, and the program prints a garbage quotient
# (-4306206720, different per build) and exits 0. A quotient that fits int64, or the same exact
# quotient printed from another function, keeps the check. Guess, unread: layer 3 works the
# quotient out in 64 bits (-2^63), so c9631fd3's "earlier sites passed" assumes -2^63 fits uint64,
# which is false, and every later site in the function is proven vacuously.
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
    if 'n' < 0 {
        emit[to.uint64[0 minus ('n' plus 1)] plus 1, true];
    } else {
        emit[to.uint64['n'], false];
    }
}

fn b1 [@x, @y] [lent @x 'a': list[uint16], lent @y 'b': list[uint16]] -> lent @x list[uint16] {
    return lend 'a';
}

fn f ['p1': uint16, 'p2': int16] -> uint16 {
    put_u[(-9223372036854775808) div.floor (-1)];
    put_i['p2' div.floor 'p1'];
    return 0;
}
MAIN { let 'x': uint16 = f[f[22, 19], -900]; }
