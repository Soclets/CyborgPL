# An enum with a case that carries data, matched case by case.
union.tag Status:
    empty;
    claimed: str.utf8;
    locked;
;union.tag Status

rec Locker:
    var number: i32;
    var status: Status;
;rec Locker

fn describe [lent $l: Locker] -> str.utf8:
    match $l.status:
        case empty:
            ret "free";
        ;case empty
        case claimed $who:
            ret "taken by " ++ $who;
        ;case claimed
        case locked:
            ret "locked";
        ;case locked
    ;match
;fn describe

MAIN:
    let $lockers := [
        Locker[number = 1, status = Status.empty],
        Locker[number = 2, status = Status.claimed("Tankun")],
        Locker[number = 3, status = Status.locked]
    ];
    for $l in lend $lockers:
        std::println("#", $l.number, ": ", describe(lend $l));
    ;for
    let $s := Status.locked;
    if $s == Status.locked:
        std::println.stored("== works on a case");
    ;if
    let $t := Status.claimed("Bo");
    std::println.stored($t);
;MAIN
