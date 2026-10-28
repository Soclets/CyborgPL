rec P:
    var n: str.utf8;
    var x: i32;
;rec P
MAIN:
    let $p := P[n = "a", x = 2];
    let $t := give $p.n;
    std::print($p.x);
;MAIN
