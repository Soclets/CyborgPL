fn f [] -> i32 or fail[a]:
    fail a;
;fn f
MAIN:
    let $x := f() + 1 ?? 0;
;MAIN
