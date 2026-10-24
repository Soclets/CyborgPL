# Tests (owner, 2026-09-29): a `test fn` takes nothing and returns nothing; `kayo test` runs each one.
# `must` states what must hold (a comparison shows both sides when it does not), `must.fail` that
# something fails (`.not_a_number`: with that reason), `must.stop` that a check site fires. Two of
# these fail on purpose. A normal run ignores the tests and runs MAIN.
fn add [$a: i32, $b: i32] -> i32 { return $a + $b; }
fn wrong [$a: i32, $b: i32] -> i32 { return $a + $b + 1; }
test fn adds_up [] {
    must add(2, 3) equal_to 5;
    must add(2, 3) > 4;
}
test fn catches_a_bug [] {
    must wrong(2, 3) equal_to 5;
}
test fn parsing [] {
    let $n := to.i32("42") otherwise 0;
    must $n equal_to 42;
    must.fail to.i32("abc");
    must.fail.not_a_number to.i32("abc");
}
test fn wrong_reason [] {
    must.fail.out_of_range to.i32("abc");
}
test fn guards [] {
    let $xs: std::list[i32] = [1, 2, 3];
    must.stop $xs[pos: 99];
}
MAIN { std::print.useful(add(1, 1), newln); }
