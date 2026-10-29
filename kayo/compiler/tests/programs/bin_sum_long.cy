# A sum of bins over more trips than Precog follows one by one (100 000)
# goes past the largest bin at about trip 180 million. Precog once let the
# sum grow forever without widening, ran out of steps, and called the add
# proven; the check is kept, and the program stops there.
%extern.C proc exit [$code: i32];
MAIN:
    let $n: i64 := 400000000;
    let $big: f64 := 10.0 std::pow 300.0;
    var $t: f64 = 0.0;
    for $i: i64 [from: 1, to: $n, by: 1]:
        change $t to $t + $big;
    ;for
    let $r: i32 := to.i32($big div.near $t) ?? 7;
    no_check my.exit($r);
;MAIN
