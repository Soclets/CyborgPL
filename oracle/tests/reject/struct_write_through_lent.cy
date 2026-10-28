rec P:
    var x: i32;
;rec P
fn poke [lent $q: P]:
    change $q.x to 9;
;fn poke
MAIN:
    let $p := P[x = 1];
    poke(lend $p);
;MAIN
