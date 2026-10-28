coll.fn greet:
    member greet [lent $who: str.utf8] -> str.utf8:
        ret $who;
    ;member greet
    member greet.loud [lent $who: str.utf8]:
        std::print($who, newln);
    ;member greet.loud
;coll.fn greet

MAIN:
    greet.loud("hi");
;MAIN
