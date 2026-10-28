# Iterators (owner, 2026-10-08/09): a type with its own `next` returning `optional[…]` is walked by
# `for`. `fn Counter.next` belongs to `Counter` and is called `Counter.next[…]` ("`b`"); `for` takes
# the iterator with `lend.mut` (it remains, used up as far as the loop went) or `give` (gone after)
# ("`c`"). `optional` is the base's: `optional.value(…)` / `optional.none`, matched bare.

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

rec Forever:
    var n: i32;
;rec Forever

proc Forever.next [lent.mut $f: Forever] -> optional[i32]:
    change $f.n to $f.n + 1;
    ret optional.value($f.n);
;proc Forever.next

MAIN:
    var $c = Counter[now = 0, stop = 5];
    match Counter.next(lend.mut $c):
        case value $x:
            std::print("by hand: ", $x, newln);
        ;case value
        case none:
            std::print("empty", newln);
        ;case none
    ;match
    for $x in lend.mut $c:
        if $x == 4:
            break;
        ;if
        std::print("lent: ", $x, newln);
    ;for
    for $x in give $c:
        std::print("given: ", $x, newln);
    ;for
    var $f = Forever[n = 0];
    for perm $y in lend.mut $f:
        if $y == 3:
            break;
        ;if
    ;for
    std::print("stopped at ", $y, newln);
;MAIN
