fn f [] -> i32 or fail[a]:
    fail a;
;fn f
fn g [] -> i32 or fail[b]:
    let $x := f()?; ret $x;
;fn g
MAIN:
    let $y := g() ?? 0;
;MAIN
