fn f [<T>: any] [let $a: <T>] -> <T>:
    ret "x";
;fn f
MAIN:
    let $x := f(1);
;MAIN
