# a function handing back a page-aligned list hands back that kind (owner, 2026-10-25)
function mk [] -> std::list.page_aligned[integer.signed.32_bit]:
    let $p: std::list[integer.signed.32_bit] = [1,]; return give $p;
end of function mk
!MAIN:
    let $a := mk();
!end of MAIN
