# A counting loop comes round into its body without asking its head again:
# the step is taken only when there is room for it before the stop
# (2026-10-15). The bounds and steps come from run time, so LLVM folds none
# of it: 'z' is 0 unless KAYO_NEVER_SET is set. Up to i64's last value 3
# rounds, down to its first 3, a u64 down to 0 is 5 3 1 (9), `continue`
# and `break` give 0 3 9 12 (24), a sieve's marking 14 8 4 2 (28), and an
# empty range none: 67.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];

MAIN {
    let $key := "KAYO_NEVER_SET";
    let mut $z: i64 = 1;
    no_check let $set := getenv(lend $key);
    if ptr.is_null($set) { change $z = 0; }
    let mut $t: i64 = 0;
    for $i: i64 [from: $z plus 9223372036854775800, to: 9223372036854775807, by: $z plus 3] { change $t = $t plus 1; }
    for $i: i64 [from: $z minus 9223372036854775800, to: -9223372036854775808, by: $z minus 4] { change $t = $t plus 1; }
    for $i: u64 [from: to.u64($z plus 5), to: 0, by: $z minus 2] { change $t = $t plus to.i64($i); }
    for $i: i64 [from: $z, to: $z plus 20, by: $z plus 3] {
        if $i equal_to $z plus 6 { continue; }
        if $i at_least $z plus 15 { break; }
        change $t = $t plus $i;
    }
    for $p: i64 [from: 2, to: $z plus 10, by: 1] {
        for $m: i64 [from: $p times $p, to: $z plus 30, by: $p] { change $t = $t plus 1; }
    }
    for $i: i64 [from: $z plus 5, to: $z plus 4, by: $z plus 1] { change $t = $t plus 100; }
    no_check my.exit(to.i32($t));
}
