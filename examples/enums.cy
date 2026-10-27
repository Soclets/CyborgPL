# An enum with a case that carries data, matched case by case.
union.tagged Status:
    empty;
    claimed: string.utf8;
    locked;
end of union.tagged Status

record Locker:
    number: integer.signed.32_bit;
    status: Status;
end of record Locker

function describe [lent $l: Locker] -> string.utf8:
    match $l.status:
        case empty:
            return "free";
        end of case empty
        case claimed $who:
            return "taken by " concatenate $who;
        end of case claimed
        case locked:
            return "locked";
        end of case locked
    end of match
end of function describe

!MAIN:
    let $lockers := [
        Locker[number = 1, status = Status.empty],
        Locker[number = 2, status = Status.claimed("Tankun")],
        Locker[number = 3, status = Status.locked]
    ];
    for $l in lend $lockers:
        std::print.shortest("#", $l.number, ": ", describe(lend $l), newline);
    end of for
    let $s := Status.locked;
    if $s equal_to Status.locked:
        std::print.stored("== works on a case", newline);
    end of if
    let $t := Status.claimed("Bo");
    std::print.stored($t, newline);
!end of MAIN
