# Iterators (owner, 2026-10-08/09): `fn Counter.next`, called by hand and
# by `for`, taken `lend.mut` (it remains) and `give` (gone after); `break`
# ends an endless one, and `perm` keeps its last item.
%extern.C proc exit [let $code: i32];
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
    var $sum: i32 = 0;
    var $c = Counter[now = 0, stop = 5];
    match Counter.next(lend.mut $c):
        case value $x:
            change $sum to $sum + $x;
        ;case value
        case none:
            change $sum to $sum + 1000;
        ;case none
    ;match
    for $x in lend.mut $c:
        if $x == 4:
            break;
        ;if
        change $sum to $sum + 10 x $x;
    ;for
    for $x in give $c:
        change $sum to $sum + 100 x $x;
    ;for
    var $f = Forever[n = 0];
    for perm $y in lend.mut $f:
        if $y == 3:
            break;
        ;if
    ;for
    change $sum to $sum + $y;
    # 1 + 20 + 30 + 500 + 3 = 554 -> exit 554 mod 256 = 42
    no_check my.exit($sum mod.floor 256);
;MAIN
