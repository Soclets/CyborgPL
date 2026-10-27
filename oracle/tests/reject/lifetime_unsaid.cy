# a function that returns a borrow says so (2026-09-29)
function noword [lent $a: string.utf8] -> string.utf8:
    return lend $a;
end of function noword
!MAIN:
    let $s := "a"; let $r := noword(lend $s);
!end of MAIN
