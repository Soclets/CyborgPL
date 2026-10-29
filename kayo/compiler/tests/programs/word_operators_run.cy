# The operators are words (owner, 2026-10-02/03): `plus`, `minus`, `times`, std's
# `pow` (above a negative, right pair first), and `.wrap`. (7 + 3) x 10 - 343
# = -243; 2 pow 3 pow 2 = 512; -2 pow 2 = -4; 2 times 3 pow 2 = 18; a u8 200
# plus.wrap 100 is 44 and times.wrap 2 is 144, 188. -243 + 512 - 4 + 18 + 188
# = 471, and 471 mod 256 is 215.
%extern.C proc exit [$code: i32];
MAIN:
    let $a: i32 := 7;
    let $b: i32 := 3;
    let $h: u8 := 200;
    let $r1 := ($a + $b) x 10 - ($a std::pow $b);
    let $r2 := 2 std::pow 3 std::pow 2;
    let $r3 := -2 std::pow 2;
    let $r4 := 2 x 3 std::pow 2;
    let $w := ($h plus.wrap 100) + ($h times.wrap 2);
    no_check my.exit(to.i32(($r1 + $r2 + $r3 + $r4 + to.i64($w)) mod.floor 256));
;MAIN
