# `before` leaves the end out (owner, 2026-10-12): 0 2 4 is 6, 5 3 1 is 9,
# an `int8` counting before 128 (which no `int8` holds) is 0 50 100, 150; a
# `uint64` from 0 before 0 runs no rounds and before 3 runs three; a slice
# before pos 4 from pos 2 holds two. 6 + 9 + 150 + 0 + 3 + 2 is 170.
extern.C fn exit ['code': int32];

fn rounds ['n': uint64] -> int64 {
    let mut 'c': int64 = 0;
    for 'i': uint64 from 0 before 'n' by 1 { change 'c' = 'c' plus 1; }
    return 'c';
}

MAIN {
    let mut 't': int64 = 0;
    for 'i' from 0 before 5 by 2 { change 't' = 't' plus 'i'; }
    for 'i' from 5 before 0 by -2 { change 't' = 't' plus 'i'; }
    for 'i': int8 from 0 before 128 by 50 { change 't' = 't' plus to.int64['i']; }
    change 't' = 't' plus rounds[0] plus rounds[3];
    let 'xs': list[int32] = [10, 20, 30, 40, 50];
    let 's' := list.slice[lend 'xs', from pos 2, before pos 4];
    change 't' = 't' plus to.int64[list.length[lend 's']];
    no_check my.exit[to.int32['t']];
}
