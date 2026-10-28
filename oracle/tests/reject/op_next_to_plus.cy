op <+> [lent $a: i32, lent $b: i32] -> i32:
    ret $a + $b;
;op <+>
MAIN:
    let $w := 1 + 2 <+> 3;
;MAIN
