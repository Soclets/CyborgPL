set Whole:
    i32;
    i64;
;set Whole
fn bigger <T: Whole> [let $a: <T>, let $b: <T>] -> <T>:
    if $a > $b:
        ret $a;
    ;if
    ret $b;
;fn bigger
MAIN:
;MAIN
