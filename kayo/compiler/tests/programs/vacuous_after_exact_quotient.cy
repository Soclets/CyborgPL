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
%extern.C proc write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;

proc emit [$m: u64, $neg: bool]:
    var $buf: std::ls[u8] = [];
    if $neg:
        std::ls.append(lend.mut $buf, value: 45);
    ;if
    var $v: u64 = $m;
    var $ds: std::ls[u8] = [];
    std::ls.append(lend.mut $ds, value: to.u8(48 + ($v mod.floor 10)));
    change $v to $v div.floor 10;
    while $v > 0:
        std::ls.append(lend.mut $ds, value: to.u8(48 + ($v mod.floor 10)));
        change $v to $v div.floor 10;
    ;while
    var $k: u64 = std::ls.len(lend $ds);
    while $k >= 1:
        std::ls.append(lend.mut $buf, value: $ds[pos: $k]);
        change $k to $k - 1;
    ;while
    std::ls.append(lend.mut $buf, value: 10);
    NO_FUCKING_CHECKS write(1, lend $buf, std::ls.len(lend $buf));
;proc emit

proc put_u [$n: u64]:
    emit($n, false);
;proc put_u

proc put_i [$n: i64]:
    if $n < 0:
        emit(to.u64(0 - ($n + 1)) + 1, true);
    else:
        emit(to.u64($n), false);
    ;if
;proc put_i

fn b1 [@x, @y] [lent @x $a: std::ls[u16], lent @y $b: std::ls[u16]] -> lent @x std::ls[u16]:
    ret lend $a;
;fn b1

proc f [$p1: u16, $p2: i16] -> u16:
    put_u((-9223372036854775808) div.floor (-1));
    put_i($p2 div.floor $p1);
    ret 0;
;proc f
MAIN:
    let $x: u16 := f(f(22, 19), -900);
;MAIN
