# Structs: building one, reading and changing its fields, an `immut` field.
record Locker:
    name: string.utf8;
    let number: integer.signed.32_bit;
    capacity: integer.signed.32_bit;
    occupied: boolean;
end of record Locker

function free_space [lent $l: Locker] -> integer.signed.32_bit:
    if $l.occupied:
        return 0;
    end of if
    return $l.capacity;
end of function free_space

procedure claim [lent.mutable $l: Locker]:
    change $l.occupied = true;
end of procedure claim

!MAIN:
    variable $mine := Locker[name = "field kit", number = 7, capacity = 12, occupied = false];
    std::print.shortest($mine.name, " is #", $mine.number, newline);
    std::print.shortest("free: ", free_space(lend $mine), newline);
    claim(lend.mutable $mine);
    std::print.shortest("free: ", free_space(lend $mine), newline);
    change $mine.capacity = 20;
    std::print.shortest($mine, newline);
    let $label := copy $mine.name;
    std::print.shortest($label, newline);
    let $taken := give $mine.name;
    std::print.shortest($taken, newline);
    let $spare: Locker = Locker[name = "spare", number = 8, capacity = 4, occupied = false];
    let $lockers := [copy $spare, Locker[name = "third", number = 9, capacity = 4, occupied = true]];
    for $l in lend $lockers:
        std::print.shortest($l.number, " ");
    end of for
    std::print.stored(newline);
!end of MAIN
