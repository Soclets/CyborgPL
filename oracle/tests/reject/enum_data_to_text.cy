union.tag Status:
    empty;
    claimed: str.utf8;
end of union.tag Status
!MAIN:
    let $s := Status.claimed("Tankun");
    let $t := to.str.utf8($s);
!end of MAIN
