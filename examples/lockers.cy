# Lockers with owners and items: structs, a constant, and lending one out to be changed.
pub const 'capacity' := 12;

struct Locker {
    immut number: i32,
    owner: str.utf8,
    items: std::list[str.utf8]
}

fn free_slots [lent 'l': Locker] -> i32 {
    return 'capacity' - std::list.length[lend 'l'.items];
}

fn stash [lent.mut 'l': Locker, 'item': str.utf8] -> bool {
    if free_slots[lend 'l'] equal_to 0 { return false; }
    std::list.append[lend.mut 'l'.items, value: give 'item'];
    return true;
}

fn label [lent 'l': Locker] -> str.utf8 {
    return "#" concat to.stored.str.utf8['l'.number] concat " (" concat 'l'.owner concat ")";
}

MAIN {
    let 'mine' := Locker[number = 7, owner = "Tankun", items = std::list.empty];
    let 'names' := ["helmet", "gloves", "rope"];
    let mut 'stored': i32 = 0;

    for 'n' in lend 'names' {
        if stash[lend.mut 'mine', copy 'n'] { change 'stored' = 'stored' + 1; }
    }
    std::print.useful[label[lend 'mine'], ": ", 'stored', " stored, ", free_slots[lend 'mine'], " free", newln];

    for 'i' [from: 1, to: std::list.length[lend 'mine'.items], by: 1] {
        check std::print.useful['i', ". ", 'mine'.items[pos: 'i'], newln];
    }

    let 'x': f64 = 0.1;
    std::print.stored['x', newln];
    std::print.useful['x', newln];

    let 'tag' := give 'mine'.owner;
    std::print.stored['tag', newln];
}
