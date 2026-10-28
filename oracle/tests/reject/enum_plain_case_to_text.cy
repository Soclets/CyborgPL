union.tag Status:
    empty;
    claimed: str.utf8;
;union.tag Status
MAIN:
    let $s := Status.empty;
    let $t := to.str.utf8(lend $s);
;MAIN
