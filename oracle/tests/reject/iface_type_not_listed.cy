set Number:
    i32;
    f64;
;set Number
fn f [<T>: Number] [$a: <T>] -> <T>:
    ret $a;
;fn f
MAIN:
    let $s := f("text");
;MAIN
