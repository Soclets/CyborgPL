rec P:
    let id: i32;
    var x: i32;
;rec P
MAIN:
    var $p = P[id = 1, x = 2];
    change $p.id to 5;
;MAIN
