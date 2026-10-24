# Finding 2026-10-15 (check run, seed 202, cut down by hand): Synapse's layer 3 proves a division
# by zero can't happen when the same function earlier printed the exact quotient
# (-9223372036854775808) div.floor (-1), which is 2^63, narrowed into u64. f[f[22, 19], -900]
# divides -900 by 0 on the second call: the oracle stops "division by zero" at that line. Synapse's
# claim for that site is "proven, layer 3, solver" (division_by_zero, past_64_bits and the i64
# narrowing all at once), its check is gone, and the program prints a garbage quotient
# (-4306206720, different per build) and exits 0. A quotient that fits i64, or the same exact
# quotient printed from another function, keeps the check. Guess, unread: layer 3 works the
# quotient out in 64 bits (-2^63), so c9631fd3's "earlier sites passed" assumes -2^63 fits u64,
# which is false, and every later site in the function is proven vacuously.
extern.C fn write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;

fn emit [$m: u64, $neg: bool] {
    let mut $buf: std::list[u8] = [];
    if $neg { std::list.append(lend.mut $buf, value: 45); }
    let mut $v: u64 = $m;
    let mut $ds: std::list[u8] = [];
    std::list.append(lend.mut $ds, value: to.u8(48 plus ($v mod.floor 10)));
    change $v = $v div.floor 10;
    while $v > 0 {
        std::list.append(lend.mut $ds, value: to.u8(48 plus ($v mod.floor 10)));
        change $v = $v div.floor 10;
    }
    let mut $k: u64 = std::list.length(lend $ds);
    while $k at_least 1 {
        std::list.append(lend.mut $buf, value: $ds[pos: $k]);
        change $k = $k minus 1;
    }
    std::list.append(lend.mut $buf, value: 10);
    NO_FUCKING_CHECKS write(1, lend $buf, std::list.length(lend $buf));
}

fn put_u [$n: u64] {
    emit($n, false);
}

fn put_i [$n: i64] {
    if $n < 0 {
        emit(to.u64(0 minus ($n plus 1)) plus 1, true);
    } else {
        emit(to.u64($n), false);
    }
}

fn b1 [@x, @y] [lent @x $a: std::list[u16], lent @y $b: std::list[u16]] -> lent @x std::list[u16] {
    return lend $a;
}

fn f [$p1: u16, $p2: i16] -> u16 {
    put_u((-9223372036854775808) div.floor (-1));
    put_i($p2 div.floor $p1);
    return 0;
}
MAIN { let $x: u16 = f(f(22, 19), -900); }
