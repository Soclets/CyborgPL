record Counter:
    now: integer.signed.32_bit; stop: integer.signed.32_bit;
end of record Counter
procedure Counter.next [lent.mutable $c: Counter] -> optional[integer.signed.32_bit]:
    if $c.now equal_to $c.stop:
        return optional.none;
    end of if
    change $c.now = $c.now plus 1;
    return optional.value($c.now);
end of procedure Counter.next
!MAIN:
    variable $c := Counter[now = 0, stop = 2];
    for $x in give $c:
    end of for
    let $again := Counter.next(lend.mutable $c);
!end of MAIN
