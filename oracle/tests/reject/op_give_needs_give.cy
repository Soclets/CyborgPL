rec V:
    var x: i32;
;rec V
op <*> [let $a: V, lent $b: i32] -> V:
    ret give $a;
;op <*>
MAIN:
    let $u := V[x = 1];
    let $w := $u <*> 2;
;MAIN
