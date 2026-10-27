# Tests (owner, 2026-09-29): a `test fn` takes nothing and returns nothing; `kayo test` runs each one.
# `must` states what must hold (a comparison shows both sides when it does not), `must.fail` that
# something fails (`.not_a_number`: with that reason), `must.stop` that a check site fires. Two of
# these fail on purpose. A normal run ignores the tests and runs MAIN.
fn add [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
    ret $a plus $b;
end of fn add
fn wrong [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
    ret $a plus $b plus 1;
end of fn wrong
test proc adds_up []:
    must add(2, 3) eq 5;
    must add(2, 3) greater_than 4;
end of test proc adds_up
test proc catches_a_bug []:
    must wrong(2, 3) eq 5;
end of test proc catches_a_bug
test proc parsing []:
    let $n := to.int.sign.32("42") otherwise 0;
    must $n eq 42;
    must.fail to.int.sign.32("abc");
    must.fail.not_a_number to.int.sign.32("abc");
end of test proc parsing
test proc wrong_reason []:
    must.fail.out_of_range to.int.sign.32("abc");
end of test proc wrong_reason
test proc guards []:
    let $xs: std::ls[int.sign.32] = [1, 2, 3];
    must.stop $xs[pos: 99];
end of test proc guards
!MAIN:
    std::print.shortest(add(1, 1), newln);
!end of MAIN
