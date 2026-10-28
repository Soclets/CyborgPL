union.tag S:
    a: i32;
    b;
;union.tag S
MAIN:
    let $s := S.a("x");
;MAIN
