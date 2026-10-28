MAIN:
    var $a: i64 = 0;
    for seq no_skip $i: i64 from 1 to 3 by 1:
        change $a to $a + $i;
    ;for
;MAIN
