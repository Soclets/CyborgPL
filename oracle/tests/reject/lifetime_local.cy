# a returned borrow cannot be of something that dies at the return (2026-09-29)
function local [] -> lent string.utf8:
    let $x := "tmp"; return lend $x;
end of function local
!MAIN:
    let $r := local();
!end of MAIN
