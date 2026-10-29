# Layer 3 asked `up['n' + 1] + 9_000_000_000_000_000_000` at 64 bits, where
# it wraps into a small number, so the landing in an `i64` read as proven
# and up(98) — past an i64 in truth — ran on with a wrong number. It is
# asked at the true value now: this must stop. 'n' is 97, and up(97) goes
# through up(98).
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
fn up [$n: i64[min: 0, max: 100]] -> i64:
    if $n >= 100:
        ret 1;
    ;if
    ret up($n + 1) + 9_000_000_000_000_000_000;
;fn up
MAIN:
    let $s := "ninety-eight bytes of text, counted by strlen at run time so nothing is known while compiling!!!!";
    no_check let $n: u64 := strlen(lend $s);
    let $k: i64[min: 0, max: 100] := to.i64($n mod.floor 101);
    no_check my.exit(to.i32(up($k) mod.floor 256));
;MAIN
