fn max [<T>: any] [$a: <T>, $b: <T>] -> <T>:
    ret $a;
;fn max
MAIN:
    let $x := max(1, "two");
;MAIN
