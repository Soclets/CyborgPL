# Tests (owner, 2026-09-29): a `test fn` takes nothing and returns nothing; `kayo test` runs each one.
# `must` states what must hold (a comparison shows both sides when it does not), `must.fail` that
# something fails (`.not_a_number`: with that reason), `must.stop` that a check site fires. Two of
# these fail on purpose. A normal run ignores the tests and runs MAIN.
fn add [let $a: i32, let $b: i32] -> i32:
    ret $a + $b;
;fn add
fn wrong [let $a: i32, let $b: i32] -> i32:
    ret $a + $b + 1;
;fn wrong
test proc adds_up []:
    must add(2, 3) == 5;
    must add(2, 3) > 4;
;test proc adds_up
test proc catches_a_bug []:
    must wrong(2, 3) == 5;
;test proc catches_a_bug
test proc parsing []:
    let $n := to.i32("42") ?? 0;
    must $n == 42;
    must.fail to.i32("abc");
    must.fail.not_a_number to.i32("abc");
;test proc parsing
test proc wrong_reason []:
    must.fail.out_of_range to.i32("abc");
;test proc wrong_reason
test proc guards []:
    let $xs: std::ls[i32] := [1, 2, 3];
    must.stop $xs[pos: 99];
;test proc guards
MAIN:
    std::println(add(1, 1));
;MAIN
