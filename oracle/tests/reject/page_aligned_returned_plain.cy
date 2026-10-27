# a function handing back a page-aligned list hands back that kind (owner, 2026-10-25)
fn mk [] -> std::ls.page_aligned[int.sign.32]:
    let $p: std::ls[int.sign.32] := [1,]; ret give $p;
end of fn mk
!MAIN:
    let $a := mk();
!end of MAIN
