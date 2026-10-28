# Lockers with owners and items: structs, a constant, and lending one out to be changed.
pub const $capacity := 12;

rec Locker:
    let number: i32;
    var owner: str.utf8;
    var items: std::ls[str.utf8];
;rec Locker

fn free_slots [lent $l: Locker] -> i32:
    ret $capacity - std::ls.len(lend $l.items);
;fn free_slots

proc stash [lent.mut $l: Locker, let $item: str.utf8] -> bool:
    if free_slots(lend $l) == 0:
        ret false;
    ;if
    std::ls.append(lend.mut $l.items, value: give $item);
    ret true;
;proc stash

fn label [lent $l: Locker] -> str.utf8:
    ret "#" ++ to.stored.str.utf8($l.number) ++ " (" ++ $l.owner ++ ")";
;fn label

MAIN:
    var $mine = Locker[number = 7, owner = "Tankun", items = std::ls.empty];
    let $names := ["helmet", "gloves", "rope"];
    var $stored: i32 = 0;

    for $n in lend $names:
        if stash(lend.mut $mine, copy $n):
            change $stored += 1;
        ;if
    ;for
    std::println(label(lend $mine), ": ", $stored, " stored, ", free_slots(lend $mine), " free");

    for $i [from: 1, to: std::ls.len(lend $mine.items), by: 1]:
        check std::println($i, ". ", $mine.items[pos: $i]);
    ;for

    let $x: f64 := 0.1;
    std::println.stored($x);
    std::println($x);

    let $tag := give $mine.owner;
    std::println.stored($tag);
;MAIN
