set Whole:
    i32;
    i64;
;set Whole
fn bigger [<T>: Whole] [$a: <T>, $b: <T>] -> <T>:
    if $a > $b:
        ret $a;
    ;if
    ret $b;
;fn bigger
MAIN:
;MAIN
