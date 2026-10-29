sig Ordered [<T>]:
    op < [lent $a: <T>, lent $b: <T>] -> bool;
;sig Ordered
rec V:
    var x: i32;
;rec V
V satisfies Ordered;
MAIN:
;MAIN
