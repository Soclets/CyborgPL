# Structs: building one, reading and changing its fields, an `immut` field.
struct Locker {
    name: str.utf8,
    immut number: int32,
    capacity: int32,
    occupied: bool
}

fn free_space [lent 'l': Locker] -> int32 {
    if 'l'.occupied {
        return 0;
    }
    return 'l'.capacity;
}

fn claim [lent.mut 'l': Locker] {
    change 'l'.occupied = true;
}

MAIN {
    let 'mine' := Locker[name = "field kit", number = 7, capacity = 12, occupied = false];
    std::print.useful['mine'.name, " is #", 'mine'.number, newln];
    std::print.useful["free: ", free_space[lend 'mine'], newln];
    claim[lend.mut 'mine'];
    std::print.useful["free: ", free_space[lend 'mine'], newln];
    change 'mine'.capacity = 20;
    std::print.useful['mine', newln];
    let 'label' := copy 'mine'.name;
    std::print.useful['label', newln];
    let 'taken' := give 'mine'.name;
    std::print.useful['taken', newln];
    let 'spare': Locker = Locker[name = "spare", number = 8, capacity = 4, occupied = false];
    let 'lockers' := [copy 'spare', Locker[name = "third", number = 9, capacity = 4, occupied = true]];
    for 'l' in lend 'lockers' {
        std::print.useful['l'.number, " "];
    }
    std::print.stored[newln];
}
