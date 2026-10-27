rec Counter:
    var now: int.sign.32;
    var stop: int.sign.32;
end of rec Counter
proc Counter.next [lent.mut $c: Counter] -> optional[int.sign.32]:
    if $c.now eq $c.stop:
        ret optional.none;
    end of if
    change $c.now to $c.now plus 1;
    ret optional.value($c.now);
end of proc Counter.next
!MAIN:
    var $c := Counter[now = 0, stop = 2];
    for $x in give $c:
    end of for
    let $again := Counter.next(lend.mut $c);
!end of MAIN
