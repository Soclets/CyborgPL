fn small [$n: i32] -> i32 or fail[bad]:
    if $n > 2:
        fail bad;
    ;if
    ret $n;
;fn small

MAIN:
    let $x := small(1) otherwise continue;
    std::print($x, newln);
;MAIN
