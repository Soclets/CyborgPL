# `pos` is removed (owner, 2026-09-27): 'x'[…] reads a list when 'x' holds
# one and calls a function when it holds one; a position can be any
# whole number, a 65-bit exact answer included. 3 + 30 + 4 = 37 (oracle).
fn inc [let $n: i32] -> i32:
    ret $n + 1;
;fn inc
fn add [let $a: i32, let $b: i32] -> i32:
    ret $a + $b;
;fn add
%extern.C proc exit [let $code: i32];
MAIN:
    let $f := inc;
    let $g := add;
    let $xs: std::ls[i32] := [4, 5, 6];
    let $a := $f[pos: 2];
    let $c := $g(10, 20);
    no_check my.exit($a + $c + $xs[pos: $xs[pos: 1] - 3]);
;MAIN
