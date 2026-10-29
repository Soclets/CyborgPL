%extern.C proc exit [let $code: i32];

MAIN:
    var $total: i32 = 0;
    for $i [from: 1, to: 10, by: 1]:
        change $total to $total + $i;
    ;for
    var $n: i32 = 0;
    while $n < 3:
        change $n to $n + 1;
    ;while
    no_check my.exit($total + $n);                      # 55 + 3 = 58
;MAIN
