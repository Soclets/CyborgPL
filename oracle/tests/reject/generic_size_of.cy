fn f [<T>: any] [let $a: <T>]:
    let $n := size.of[<T>];
;fn f
MAIN:
    f(1);
;MAIN
