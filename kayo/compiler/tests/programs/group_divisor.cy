# Bare numbers alone are worked out exactly (dc83c05), beside `div` and
# `mod` too (check claude, seed 4698): the group took the typed side's
# `u8`, and its `221 x 2` was refused as 442. 7 mod 3 is 1, 7 div 3 is
# 2, 12 mod 7 is 5: 1 + 20 + 5 x 10 = 71.
%extern.C proc exit [$code: i32];
MAIN:
    let $p: u8 := 7;
    let $a: u8 := $p mod.floor (((221 x 2) mod.floor 5) + 1);
    let $b: u8 := $p div.floor (((221 x 2) mod.floor 5) + 1);
    let $c: u8 := (((221 x 2) mod.floor 5) + 10) mod.floor $p;
    no_check my.exit(to.i32($a) + to.i32($b) x 10 + to.i32($c) x 10);
;MAIN
