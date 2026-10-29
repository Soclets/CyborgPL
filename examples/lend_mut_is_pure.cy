# Changing what a function was lent is no effect (owner, 2026-10-29: "(a)"): a `lend.mut` hands the value over and takes
# the changed one back, and nothing else can see it meanwhile, so `append_twice` and `count_up` are functions, and a
# function may call them on its own locals, or pass on what it was lent itself.
fn append_twice [lent.mut $out: std::ls[i32], $v: i32]:
    std::ls.append(lend.mut $out, value: $v);
    std::ls.append(lend.mut $out, value: $v);
;fn append_twice

rec Counter:
    var n: i32;
;rec Counter

fn count_up [lent.mut $c: Counter]:
    change $c.n to $c.n + 1;
;fn count_up

fn doubled [$a: i32, $b: i32] -> std::ls[i32]:
    var $xs: std::ls[i32] = [];
    append_twice(lend.mut $xs, $a);
    append_twice(lend.mut $xs, $b);
    ret give $xs;
;fn doubled

fn ticks [$times: i32] -> i32:
    var $c = Counter[n = 0];
    for $i [from: 1, to: $times, by: 1]:
        count_up(lend.mut $c);
    ;for
    ret $c.n;
;fn ticks

MAIN:
    std::print(doubled(3, 5), " ", ticks(4), newln);
;MAIN
