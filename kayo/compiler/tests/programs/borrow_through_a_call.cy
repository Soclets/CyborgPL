# A function hands back the borrow another call handed it (check claude,
# 2026-09-29, 46faa74): `second`'s result borrows its own lent 'a', through
# `first`. The oracle prints 2 for the same program.
%extern.C proc exit [let $code: i32];
fn first [lent $a: std::ls[i32]] -> lent std::ls[i32]:
    ret lend $a;
;fn first
fn second [lent $a: std::ls[i32]] -> lent std::ls[i32]:
    ret first(lend $a);
;fn second
MAIN:
    let $s: std::ls[i32] := [1, 2, 3];
    let $r := second(lend $s);
    no_check my.exit($r[pos: 2]);
;MAIN
