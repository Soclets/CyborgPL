sig Ordered [<T>]:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
;sig Ordered
fn max [<T>: Ordered] [let $a: <T>, let $b: <T>] -> <T>:
    if $b < $a:
        ret $a;
    ;if
    ret $b;
;fn max
MAIN:
    let $x := max("a", "b");
;MAIN
