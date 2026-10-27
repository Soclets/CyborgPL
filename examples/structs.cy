# Structs: building one, reading and changing its fields, an `immut` field.
rec Locker:
    name: str.utf8;
    let number: int.sign.32;
    capacity: int.sign.32;
    occupied: bool;
end of rec Locker

fn free_space [lent $l: Locker] -> int.sign.32:
    if $l.occupied:
        ret 0;
    end of if
    ret $l.capacity;
end of fn free_space

proc claim [lent.mut $l: Locker]:
    change $l.occupied = true;
end of proc claim

!MAIN:
    var $mine := Locker[name = "field kit", number = 7, capacity = 12, occupied = false];
    std::print.shortest($mine.name, " is #", $mine.number, newln);
    std::print.shortest("free: ", free_space(lend $mine), newln);
    claim(lend.mut $mine);
    std::print.shortest("free: ", free_space(lend $mine), newln);
    change $mine.capacity = 20;
    std::print.shortest($mine, newln);
    let $label := copy $mine.name;
    std::print.shortest($label, newln);
    let $taken := give $mine.name;
    std::print.shortest($taken, newln);
    let $spare: Locker = Locker[name = "spare", number = 8, capacity = 4, occupied = false];
    let $lockers := [copy $spare, Locker[name = "third", number = 9, capacity = 4, occupied = true]];
    for $l in lend $lockers:
        std::print.shortest($l.number, " ");
    end of for
    std::print.stored(newln);
!end of MAIN
