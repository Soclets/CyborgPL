# two meanings of one operator for the same types (2026-09-23)
rec A:
    var x: i32;
;rec A
op <+> [lent $a: A, lent $b: A] -> i32:
    ret 1;
;op <+>
op <+> [lent $a: A, lent $b: A] -> i32:
    ret 2;
;op <+>
MAIN:
;MAIN
