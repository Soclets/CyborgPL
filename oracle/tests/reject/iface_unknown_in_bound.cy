fn f [<T>: Nope] [$a: <T>] -> <T>:
    ret $a;
;fn f
MAIN:
    let $x := f(1);
;MAIN
