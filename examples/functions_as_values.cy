# Functions as values (design/functions-as-values.md, 2026-09-21).
struct Locker { number: i32, owner: str.utf8 }

fn by_number [lent 'a': Locker, lent 'b': Locker] -> bool { return 'a'.number < 'b'.number; }
fn by_owner  [lent 'a': Locker, lent 'b': Locker] -> bool { return std::str.bytes[lend 'a'.owner] < std::str.bytes[lend 'b'.owner]; }

# a function that takes a function: the type is the signature without the names
fn first_by [lent 'xs': std::list[Locker], 'less': fn[lent Locker, lent Locker] -> bool] -> str.utf8 {
    let 'best' := 1;
    for 'i' [from: 2, to: std::list.length[lend 'xs'], by: 1] {
        if 'less'[lend 'xs'[pos: 'i'], lend 'xs'[pos: 'best']] { change 'best' = 'i'; }
    }
    let 'winner' := lend 'xs'[pos: 'best'];
    return copy 'winner'.owner;
}

fn keep [lent 'xs': std::list[i32], 'test': fn[i32] -> bool] -> std::list[i32] {
    let 'out': std::list[i32] = [];
    for 'x' in lend 'xs' { if 'test'['x'] { std::list.append[lend.mut 'out', value: 'x']; } }
    return give 'out';
}

MAIN {
    let 'lockers' := [Locker[number = 7, owner = "Cy"], Locker[number = 2, owner = "Ann"], Locker[number = 5, owner = "Bo"]];
    std::print.stored[first_by[lend 'lockers', by_number], newln];          # a named function, as a value: its bare name
    std::print.stored[first_by[lend 'lockers', by_owner], newln];

    let 'f' := by_number;                                                 # held in a variable
    std::print.useful['f'[lend 'lockers'[pos: 1], lend 'lockers'[pos: 2]], newln];   # called: 'f'[…]

    let 'limit' := 60;
    let 'scores' := [72, 40, 95, 55];
    let 'passing' := keep[lend 'scores', fn ['s': i32] -> bool with [lend 'limit'] { return 's' at_least 'limit'; }];
    std::print.useful['passing', newln];

    let 'evens' := keep[lend 'scores', fn ['s': i32] -> bool { return ('s' mod.floor 2) equal_to 0; }];   # captures nothing
    std::print.useful['evens', newln];
}
