fn f [<T>: Nope] [let $a: <T>] -> <T>:
    ret $a;
;fn f
MAIN:
    let $x := f(1);
;MAIN
