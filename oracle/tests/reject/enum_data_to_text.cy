union.tag Status:
    empty;
    claimed: str.utf8;
;union.tag Status
MAIN:
    let $s := Status.claimed("Tankun");
    let $t := to.str.utf8($s);
;MAIN
