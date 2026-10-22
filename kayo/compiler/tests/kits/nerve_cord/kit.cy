# A Nerve cord for the tests (owner, 2026-10-14): a Neuron that lists it by cord's ID and a `path` takes
# over `std::` for the whole program, name by name. It lists no cord, and lists are its own.

pub fn twice ['x': i64] -> i64 {
    return 'x' times 2;
}

pub fn total [lent 'xs': std::list[i64]] -> i64 {
    let mut 'sum': i64 = 0;
    for 'x' in lend 'xs' { change 'sum' = 'sum' plus 'x'; }
    return 'sum';
}

# Taking over the built-in `std::exit`: this one does not stop the program.
pub fn exit ['code': u8] {
}
