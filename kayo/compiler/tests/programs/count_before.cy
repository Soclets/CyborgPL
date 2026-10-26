# `before` leaves the end out (owner, 2026-10-12): 0 2 4 is 6, 5 3 1 is 9,
# an `i8` counting before 128 (which no `i8` holds) is 0 50 100, 150; a
# `u64` from 0 before 0 runs no rounds and before 3 runs three; a slice
# before pos 4 from pos 2 holds two. 6 + 9 + 150 + 0 + 3 + 2 is 170.
%extern.C fn exit [$code: i32];

fn rounds [$n: u64] -> i64 {
    let mut $c: i64 = 0;
    for $i: u64 [from: 0, before: $n, by: 1] { change $c = $c plus 1; }
    return $c;
}

MAIN {
    let mut $t: i64 = 0;
    for $i [from: 0, before: 5, by: 2] { change $t = $t plus $i; }
    for $i [from: 5, before: 0, by: -2] { change $t = $t plus $i; }
    for $i: i8 [from: 0, before: 128, by: 50] { change $t = $t plus to.i64($i); }
    change $t = $t plus rounds(0) plus rounds(3);
    let $xs: std::list[i32] = [10, 20, 30, 40, 50];
    let $s := std::list.slice(lend $xs, from: pos 2, before: pos 4);
    change $t = $t plus to.i64(std::list.length(lend $s));
    no_check my.exit(to.i32($t));
}
