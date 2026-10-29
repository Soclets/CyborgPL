fn f [<T>: any] [$a: <T>]:
    let $n := size.of[<T>];
;fn f
MAIN:
    f(1);
;MAIN
