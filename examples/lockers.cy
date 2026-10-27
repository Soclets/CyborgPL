# Lockers with owners and items: structs, a constant, and lending one out to be changed.
public constant $capacity := 12;

record Locker:
    let number: integer.signed.32_bit;
    owner: string.utf8;
    items: std::list[string.utf8];
end of record Locker

function free_slots [lent $l: Locker] -> integer.signed.32_bit:
    return $capacity minus std::list.length(lend $l.items);
end of function free_slots

procedure stash [lent.mutable $l: Locker, $item: string.utf8] -> boolean:
    if free_slots(lend $l) equal_to 0:
        return false;
    end of if
    std::list.append(lend.mutable $l.items, value: give $item);
    return true;
end of procedure stash

function label [lent $l: Locker] -> string.utf8:
    return "#" concatenate to.stored.string.utf8($l.number) concatenate " (" concatenate $l.owner concatenate ")";
end of function label

!MAIN:
    variable $mine := Locker[number = 7, owner = "Tankun", items = std::list.empty];
    let $names := ["helmet", "gloves", "rope"];
    variable $stored: integer.signed.32_bit = 0;

    for $n in lend $names:
        if stash(lend.mutable $mine, copy $n):
            change $stored = $stored plus 1;
        end of if
    end of for
    std::print.shortest(label(lend $mine), ": ", $stored, " stored, ", free_slots(lend $mine), " free", newline);

    for $i [from: 1, to: std::list.length(lend $mine.items), by: 1]:
        check std::print.shortest($i, ". ", $mine.items[position: $i], newline);
    end of for

    let $x: float.binary.64_bit = 0.1;
    std::print.stored($x, newline);
    std::print.shortest($x, newline);

    let $tag := give $mine.owner;
    std::print.stored($tag, newline);
!end of MAIN
