rec Counter:
    var now: i32;
    var stop: i32;
;rec Counter
proc Counter.next [lent.mut $c: Counter] -> optional[i32]:
    if $c.now == $c.stop:
        ret optional.none;
    ;if
    change $c.now += 1;
    ret optional.value($c.now);
;proc Counter.next
MAIN:
    var $c = Counter[now = 0, stop = 2];
    for $x in give $c:
    ;for
    let $again := Counter.next(lend.mut $c);
;MAIN
