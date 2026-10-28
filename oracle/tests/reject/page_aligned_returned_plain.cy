# a function handing back a page-aligned list hands back that kind (owner, 2026-10-25)
fn mk [] -> std::ls.page_aligned[i32]:
    let $p: std::ls[i32] := [1,]; ret give $p;
;fn mk
MAIN:
    let $a := mk();
;MAIN
