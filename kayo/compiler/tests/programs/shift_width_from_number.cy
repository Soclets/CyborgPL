# A shift's width is the shifted number's own type, or where the value goes,
# never its count (owner, 2026-10-07): ((0 minus 53) shr 'c') mod.floor 23 into
# an i64 is a 64-bit shift, 22; 7 shr (('a' minus 1) mod.floor 32) into an
# i64 is 7 shr 0, 7. 22 + 7 = 29, as the oracle has it.
%extern.C proc exit [let $code: i32];
MAIN:
    let $c: i8 := 9;
    let $r: i64 := ((0 - 53) shr $c) mod.floor 23;
    let $a: i8 := 33;
    let $s: i64 := 7 shr (($a - 1) mod.floor 32);
    no_check my.exit(to.i32($r + $s));
;MAIN
