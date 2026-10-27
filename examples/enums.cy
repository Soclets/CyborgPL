# An enum with a case that carries data, matched case by case.
union.tag Status:
    empty;
    claimed: str.utf8;
    locked;
end of union.tag Status

rec Locker:
    number: int.sign.32;
    status: Status;
end of rec Locker

fn describe [lent $l: Locker] -> str.utf8:
    match $l.status:
        case empty:
            ret "free";
        end of case empty
        case claimed $who:
            ret "taken by " concat $who;
        end of case claimed
        case locked:
            ret "locked";
        end of case locked
    end of match
end of fn describe

!MAIN:
    let $lockers := [
        Locker[number = 1, status = Status.empty],
        Locker[number = 2, status = Status.claimed("Tankun")],
        Locker[number = 3, status = Status.locked]
    ];
    for $l in lend $lockers:
        std::print.shortest("#", $l.number, ": ", describe(lend $l), newln);
    end of for
    let $s := Status.locked;
    if $s eq Status.locked:
        std::print.stored("== works on a case", newln);
    end of if
    let $t := Status.claimed("Bo");
    std::print.stored($t, newln);
!end of MAIN
