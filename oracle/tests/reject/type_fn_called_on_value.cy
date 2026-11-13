# A function that belongs to a type is called through the type, `Counter.next(lend.mut $c)`; there is no `$c.next()` (owner, 2026-10-09: "`b`").
rec Counter:
    var n: i32;
;rec Counter
fn Counter.next [lent.mut $c: Counter]:
    change $c.n to $c.n + 1;
;fn Counter.next
MAIN:
    var $c = Counter[n = 0];
    $c.next();
;MAIN
