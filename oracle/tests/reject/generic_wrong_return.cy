fn f [<T>: any] [$a: <T>] -> <T>:
    ret "x";
;fn f
MAIN:
    let $x := f(1);
;MAIN
