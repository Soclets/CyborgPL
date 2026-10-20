# One name, several versions, picked by how many arguments and their types (owner, 2026-10-20),
# written together in a `coll.fn` (owner: b). A version is the own function or a member written again.
coll.fn describe {
    describe ['n': int64] -> str.utf8 { return "a whole number"; }
    describe ['x': bin64] -> str.utf8 { return "a binary float"; }
    describe [lent 's': str.utf8] -> str.utf8 { return "a text"; }
}

coll.fn area {
    area.rect ['side': int64] -> int64 { return 'side' x 'side'; }
    area.rect ['w': int64, 'h': int64] -> int64 { return 'w' x 'h'; }
}

MAIN {
    let 'n': int64 = 7;
    let 'x': bin64 = 2.5;
    let 'name' := "Ann";
    std::print.useful[describe['n'], newln];
    std::print.useful[describe['x'], newln];
    std::print.useful[describe[lend 'name'], newln];
    std::print.useful[area.rect[3], " ", area.rect[3, 4], newln];
}
