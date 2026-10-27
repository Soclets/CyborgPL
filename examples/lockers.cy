# Lockers with owners and items: structs, a constant, and lending one out to be changed.
pub const $capacity := 12;

rec Locker:
    let number: int.sign.32;
    var owner: str.utf8;
    var items: std::ls[str.utf8];
end of rec Locker

fn free_slots [lent $l: Locker] -> int.sign.32:
    ret $capacity minus std::ls.len(lend $l.items);
end of fn free_slots

proc stash [lent.mut $l: Locker, let $item: str.utf8] -> bool:
    if free_slots(lend $l) eq 0:
        ret false;
    end of if
    std::ls.append(lend.mut $l.items, value: give $item);
    ret true;
end of proc stash

fn label [lent $l: Locker] -> str.utf8:
    ret "#" concat to.stored.str.utf8($l.number) concat " (" concat $l.owner concat ")";
end of fn label

!MAIN:
    var $mine := Locker[number = 7, owner = "Tankun", items = std::ls.empty];
    let $names := ["helmet", "gloves", "rope"];
    var $stored: int.sign.32 := 0;

    for $n in lend $names:
        if stash(lend.mut $mine, copy $n):
            change $stored to $stored plus 1;
        end of if
    end of for
    std::print.shortest(label(lend $mine), ": ", $stored, " stored, ", free_slots(lend $mine), " free", newln);

    for $i [from: 1, to: std::ls.len(lend $mine.items), by: 1]:
        check std::print.shortest($i, ". ", $mine.items[pos: $i], newln);
    end of for

    let $x: float.bin.64 := 0.1;
    std::print.stored($x, newln);
    std::print.shortest($x, newln);

    let $tag := give $mine.owner;
    std::print.stored($tag, newln);
!end of MAIN
