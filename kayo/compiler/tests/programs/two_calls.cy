%extern.C proc exit [let $code: i32];
rec P:
    var x: i32;
;rec P
fn get [let $p: P] -> i32:
    ret $p.x;
;fn get
MAIN:
    let $a := P[x = 5];
    let $b := P[x = 9];
    no_check my.exit(get(give $a) + get(give $b));
;MAIN
