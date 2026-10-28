fn f [] -> i32:
    ret 1;
;fn f
MAIN:
    let $x := f() ?? 0;
;MAIN
