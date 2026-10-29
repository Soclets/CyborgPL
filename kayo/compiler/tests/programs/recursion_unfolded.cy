# kayo inlines a function that calls itself into itself a few levels
# (owner, 2026-10-17): what each computes stays the same. The order of
# effects (a walk writing before and after its call), a failure climbing
# out from deep down, two calls with different arguments, recursion far
# deeper than the levels unfolded, and two functions calling each other.
%extern.C proc exit [$code: i32];

fn fib [$n: i64[min: 0, max: 92]] -> i64:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib

fn walk [$n: i32[min: 0, max: 6]] -> str.utf8:
    if $n == 0:
        ret str.empty;
    ;if
    ret to.stored.str.utf8($n) concat walk($n - 1) concat to.stored.str.utf8(10 + $n);
;fn walk

fn depth [$n: i32[min: 0, max: 5000]] -> i32:
    if $n == 0:
        ret 0;
    ;if
    ret depth($n - 1) + 1;
;fn depth

fn find [$n: i32[min: 0, max: 50]] -> i32 or fail[missing]:
    if $n == 0:
        fail missing;
    ;if
    if $n == 7:
        ret 7;
    ;if
    let $r := find($n - 1)?;
    ret $r + 1;
;fn find

fn ack [$m: i64[min: 0, max: 2], $n: i64[min: 0, max: 20]] -> i64:
    if $m == 0:
        ret $n + 1;
    ;if
    if $n == 0:
        ret ack($m - 1, 1);
    ;if
    let $inner := ack($m, $n - 1);
    if $inner > 20:
        ret 0;
    ;if
    ret ack($m - 1, $inner);
;fn ack

fn even [$n: i32[min: 0, max: 5000]] -> bool:
    if $n == 0:
        ret true;
    ;if
    ret odd($n - 1);
;fn even

fn odd [$n: i32[min: 0, max: 5000]] -> bool:
    if $n == 0:
        ret false;
    ;if
    ret even($n - 1);
;fn odd

MAIN:
    var $got: i32 = 0;
    if fib(30) == 832040:
        change $got to $got + 1;
    ;if
    if walk(4) == "432111121314":
        change $got to $got + 2;
    ;if
    if depth(4000) == 4000:
        change $got to $got + 4;
    ;if
    let $a := find(20) ?? -1;
    let $b := find(5) ?? -1;
    if $a == 20 and $b == -1:
        change $got to $got + 8;
    ;if
    if ack(2, 3) == 9:
        change $got to $got + 16;
    ;if
    if even(4000) and odd(3999) and not even(777):
        change $got to $got + 32;
    ;if
    no_check my.exit($got);
;MAIN
