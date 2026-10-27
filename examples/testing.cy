# Tests (owner, 2026-09-29): a `test fn` takes nothing and returns nothing; `kayo test` runs each one.
# `must` states what must hold (a comparison shows both sides when it does not), `must.fail` that
# something fails (`.not_a_number`: with that reason), `must.stop` that a check site fires. Two of
# these fail on purpose. A normal run ignores the tests and runs MAIN.
function add [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a plus $b;
end of function add
function wrong [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a plus $b plus 1;
end of function wrong
test procedure adds_up []:
    must add(2, 3) equal_to 5;
    must add(2, 3) greater_than 4;
end of test procedure adds_up
test procedure catches_a_bug []:
    must wrong(2, 3) equal_to 5;
end of test procedure catches_a_bug
test procedure parsing []:
    let $n := to.integer.signed.32_bit("42") otherwise 0;
    must $n equal_to 42;
    must.fail to.integer.signed.32_bit("abc");
    must.fail.not_a_number to.integer.signed.32_bit("abc");
end of test procedure parsing
test procedure wrong_reason []:
    must.fail.out_of_range to.integer.signed.32_bit("abc");
end of test procedure wrong_reason
test procedure guards []:
    let $xs: std::list[integer.signed.32_bit] = [1, 2, 3];
    must.stop $xs[position: 99];
end of test procedure guards
!MAIN:
    std::print.shortest(add(1, 1), newline);
!end of MAIN
