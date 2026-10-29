# Runs to its end on a bare chip (owner, 2026-10-09: `aarch64-none`), so
# the CPU sleeps in the entry. A wrong answer would take a u8 below 0, a
# failed check, and freeze it on a trap instead: there is no exit code to
# carry the answer, so the trap is the test.
fn fib [let $n: i64] -> i64:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib

rec Counter:
    var now: i32;
    var stop: i32;
;rec Counter

proc Counter.next [lent.mut $c: Counter] -> optional[i32]:
    if $c.now == $c.stop:
        ret optional.none;
    ;if
    change $c.now to $c.now + 1;
    ret optional.value($c.now);
;proc Counter.next

MAIN:
    var $wrong: u8 = 0;
    if fib(24) != 46368:
        change $wrong to $wrong - 1;
    ;if
    let $half: f64 := 3.0 x 0.5;
    if $half != 1.5:
        change $wrong to $wrong - 1;
    ;if
    var $sum: i32 = 0;
    var $c = Counter[now = 0, stop = 10];
    for $x in lend.mut $c:
        change $sum to $sum + $x;
    ;for
    if $sum != 55:
        change $wrong to $wrong - 1;
    ;if
;MAIN
