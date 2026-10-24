# "Bare numbers alone are worked out exactly at compile time" (operators.md),
# `/` and `mod` too, with no 64-bit window in between: only where the answer
# lands is checked. (2^64 - 1 + 1) mod 1 is 0; it was refused, in a loop and
# out of one (check claude, attack a: 2026-09-24-bare-sum-past-64-in-loop).
extern.C fn exit [$code: i32];
MAIN {
    let mut $v: u64 = 9;
    let mut $w: u16 = 2;
    loop {
        if $w equal_to 0 { break; }
        change $w = $w minus 1;
        change $v = (18446744073709551615 plus 1) mod.floor 1;
    }
    let $u: u64 = (18446744073709551615 times 4) div.floor 2 minus 18446744073709551615 minus 1;
    let $t: i8 = ((1000 times 1000) mod.floor 7) minus (200 plus 100) div.floor 3;
    no_check my.exit(to.i32($v) plus to.i32($u mod.floor 256) plus to.i32($t) plus 100);
}
