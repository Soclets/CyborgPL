# The whole struct replaced: the guard on its old field says nothing.
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
rec Clock:
    var now: i32[min: 0, max: 2895];
    var stop: i32[min: 0, max: 2880];
;rec Clock

proc bump [lent.mut $c: Clock]:
    change $c.now to 2895;
;proc bump

proc Clock.next [lent.mut $c: Clock] -> optional[i32]:
    if $c.now >= 2880:
        ret optional.none;
    ;if
    change $c to Clock[now = 2890, stop = 0];
    change $c.now to $c.now + 15;
    ret optional.none;
;proc Clock.next

MAIN:
    let $k := "N";
    no_check let $n: i32 := atoi(getenv(lend $k));
    if $n >= 0 and $n <= 2880:
        var $quarters = Clock[now = $n, stop = $n];
        for $t in lend.mut $quarters:
            std::print(" ", $t);
        ;for
    ;if
;MAIN
