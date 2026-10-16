# An enum with a case that carries data, matched case by case.
enum Status {
    empty,
    claimed: str.utf8,
    locked
}

struct Locker {
    number: int32,
    status: Status
}

fn describe [lent 'l': Locker] -> str.utf8 {
    match 'l'.status {
        empty         { return "free"; }
        claimed 'who' { return "taken by " concat 'who'; }
        locked        { return "locked"; }
    }
}

MAIN {
    let 'lockers' := [
        Locker[number = 1, status = Status.empty],
        Locker[number = 2, status = Status.claimed["Tankun"]],
        Locker[number = 3, status = Status.locked]
    ];
    for 'l' in lend 'lockers' {
        std::print.useful["#", 'l'.number, ": ", describe[lend 'l'], newln];
    }
    let 's' := Status.locked;
    if 's' equal_to Status.locked { std::print.stored["== works on a case", newln]; }
    let 't' := Status.claimed["Bo"];
    std::print.stored['t', newln];
}
