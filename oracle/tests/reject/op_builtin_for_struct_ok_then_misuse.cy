rec V:
    var x: i32;
;rec V
op + [lent $a: V, lent $b: V] -> V:
    ret V[x = $a.x + $b.x];
;op +
MAIN:
    let $u := V[x = 1];
    let $w := $u + 1;
;MAIN
