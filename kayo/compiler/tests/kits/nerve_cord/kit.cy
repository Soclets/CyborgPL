# A Nerve cord for the tests (owner, 2026-10-14): a Neuron that lists it by cord's ID and a `path` takes
# over `std::` for the whole program, name by name. It lists no cord, and lists are its own.

pub fn twice [$x: i64] -> i64:
    ret $x x 2;
;fn twice

pub fn total [lent $xs: std::ls[i64]] -> i64:
    var $sum: i64 = 0;
    for $x in lend $xs:
        change $sum to $sum + $x;
    ;for
    ret $sum;
;fn total

# Taking over the built-in `std::exit`: this one does not stop the program.
pub fn exit [$code: u8]:
;fn exit
