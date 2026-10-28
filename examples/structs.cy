# Structs: building one, reading and changing its fields, an `immut` field.
rec Locker:
    var name: str.utf8;
    let number: i32;
    var capacity: i32;
    var occupied: bool;
;rec Locker

fn free_space [lent $l: Locker] -> i32:
    if $l.occupied:
        ret 0;
    ;if
    ret $l.capacity;
;fn free_space

proc claim [lent.mut $l: Locker]:
    change $l.occupied to true;
;proc claim

MAIN:
    var $mine = Locker[name = "field kit", number = 7, capacity = 12, occupied = false];
    std::print($mine.name, " is #", $mine.number, newln);
    std::print("free: ", free_space(lend $mine), newln);
    claim(lend.mut $mine);
    std::print("free: ", free_space(lend $mine), newln);
    change $mine.capacity to 20;
    std::print($mine, newln);
    let $label := copy $mine.name;
    std::print($label, newln);
    let $taken := give $mine.name;
    std::print($taken, newln);
    let $spare: Locker := Locker[name = "spare", number = 8, capacity = 4, occupied = false];
    let $lockers := [copy $spare, Locker[name = "third", number = 9, capacity = 4, occupied = true]];
    for $l in lend $lockers:
        std::print($l.number, " ");
    ;for
    std::print.stored(newln);
;MAIN
