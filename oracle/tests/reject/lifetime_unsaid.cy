# a function that returns a borrow says so (2026-09-29)
fn noword [lent $a: str.utf8] -> str.utf8:
    ret lend $a;
end of fn noword
!MAIN:
    let $s := "a"; let $r := noword(lend $s);
!end of MAIN
