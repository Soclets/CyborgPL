%extern.C proc exit [$code: i32];
fn half [$n: i32] -> i32 or fail[odd, negative: i32]:
    if $n < 0:
        fail negative[$n];
    ;if
    if ($n mod.floor 2) == 1:
        fail odd;
    ;if
    ret $n div.floor 2;
;fn half
MAIN:
    let $a := half(10) ?? 0;
    let $b := half(7) ?? 100;
    var $c: i32 = 0;
    half(-4)-
    otherwise:
        case odd:
            change $c to 1;
        ;case odd
        case negative $v:
            change $c to 0 - $v;
        ;case negative
    ;otherwise;
    no_check my.exit($a + $b + $c);
;MAIN
