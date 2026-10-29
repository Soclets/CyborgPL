sig O [<T>]:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
;sig O
fn f [<T>: any and O] [$a: <T>] -> <T>:
    ret $a;
;fn f
MAIN:
;MAIN
