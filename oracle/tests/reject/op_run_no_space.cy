op <+> [lent $a: i32, lent $b: i32] -> i32:
    ret $a + $b;
;op <+>
MAIN:
    let $b := 2;
    let $w := 1 <+>-$b;
;MAIN
