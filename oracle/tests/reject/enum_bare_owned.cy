union.tag S:
    a;
    b: i32;
;union.tag S
MAIN:
    let $s := S.a;
    let $t := $s;
;MAIN
