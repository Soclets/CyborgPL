module a:
    rec P:
        var x: i32;
    ;rec P
;module a
MAIN:
    let $p := a.P[x = 1];
;MAIN
