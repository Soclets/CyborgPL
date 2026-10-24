# Every division says how it rounds (owner, 2026-10-02), worked out at run
# time: the values pass through C, so nothing is folded. `floor` down,
# `ceil` up, `trunc` toward zero, `near` to the nearest with a tie to even;
# `mod.X` is what `div.X` leaves. On a bin or deci `div.near` is ordinary
# division and the others give a whole quotient. The smallest i8 by -1
# is 128, exact where it lands. Each right group adds its own bit: 255.
extern.C fn exit [$code: i32];
extern.C fn strlen [$s: ptr[u8]] -> u64;
MAIN {
    let $s := "x";
    no_check let $n: u64 = strlen(lend $s);
    let $z: i32 = to.i32($n) minus 1;
    let $a: i32 = -7 plus $z;
    let $b: i32 = 2 plus $z;
    let $c: i32 = 5 plus $z;
    let mut $code: i32 = 0;
    if (($a div.floor $b) equal_to -4) and (($a div.ceil $b) equal_to -3) and (($a div.trunc $b) equal_to -3) and (($a div.near $b) equal_to -4) { change $code = $code plus 1; }
    if (($a mod.floor $b) equal_to 1) and (($a mod.ceil $b) equal_to -1) and (($a mod.trunc $b) equal_to -1) and (($a mod.near $b) equal_to 1) { change $code = $code plus 2; }
    if (($c div.near $b) equal_to 2) and (($c mod.near $b) equal_to 1) and ((7 div.near $b) equal_to 4) { change $code = $code plus 4; }
    let $u: u32 = to.u32(7 plus $z);
    let $v: u32 = to.u32(2 plus $z);
    if (($u div.ceil $v) equal_to 4) and (($u div.near $v) equal_to 4) and (($u div.floor $v) equal_to 3) { change $code = $code plus 8; }
    let $lo: i8 = to.i8(-128 plus $z);
    let $m1: i8 = to.i8(-1 plus $z);
    let $big: i16 = $lo div.trunc $m1;
    if ($big equal_to 128) and (($lo mod.near $m1) equal_to 0) { change $code = $code plus 16; }
    let $one: f64 = to.f64(1 plus $z);
    let $x: f64 = 7.5 times $one;
    let $y: f64 = 2.0 times $one;
    if (($x div.near $y) equal_to 3.75) and (($x div.floor $y) equal_to 3.0) and (($x mod.floor $y) equal_to 1.5) and (($x div.ceil $y) equal_to 4.0) and (($x mod.ceil $y) equal_to -0.5) { change $code = $code plus 32; }
    let $h: brain16 = to.brain16(-5 plus $z);
    let $k: brain16 = to.brain16(3 plus $z);
    if (($h div.floor $k) equal_to -2.0) and (($h mod.floor $k) equal_to 1.0) and (($h div.trunc $k) equal_to -1.0) and (($h mod.trunc $k) equal_to -2.0) { change $code = $code plus 64; }
    let $p: d64 = -7.5 plus to.d64($z);
    let $q: d64 = 2.0 plus to.d64($z);
    if (($p div.near $q) equal_to -3.75) and (($p div.floor $q) equal_to -4) and (($p mod.floor $q) equal_to 0.5) and (($p div.trunc $q) equal_to -3) and (($p mod.trunc $q) equal_to -1.5) and (($p mod.ceil $q) equal_to -1.5) { change $code = $code plus 128; }
    no_check my.exit($code);
}
