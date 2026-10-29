# old_spelling: an operator declared with a retired symbol.
rec V:
    var n: i32;
;rec V
op <= [lent $a: V, lent $b: V] -> bool:
    ret $a.n < $b.n;
;op <=
MAIN:
;MAIN
