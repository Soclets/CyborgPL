rec P:
    var n: str.utf8;
;rec P
MAIN:
    let $p := P[n = "a"];
    let $t := $p.n;
;MAIN
