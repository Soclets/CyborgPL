union.tagged Status:
    empty;
    claimed: string.utf8;
end of union.tagged Status
!MAIN:
    let $s := Status.claimed("Tankun");
    let $t := to.string.utf8($s);
!end of MAIN
