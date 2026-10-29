fn max [<T>: any] [let $a: <T>, let $b: <T>] -> <T>:
    ret $a;
;fn max
MAIN:
    let $x := max(1, "two");
;MAIN
