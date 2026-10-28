op dot [lent $a: i32, lent $b: i32] -> i32:
    ret $a x $b;
;op dot
MAIN:
    let $x := dot(1, 2);
;MAIN
