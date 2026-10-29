# A counting loop comes round into its body without asking its head again:
# the step is taken only when there is room for it before the stop
# (2026-10-15). The bounds and steps come from run time, so LLVM folds none
# of it: 'z' is 0 unless KAYO_NEVER_SET is set. Up to i64's last value 3
# rounds, down to its first 3, a u64 down to 0 is 5 3 1 (9), `continue`
# and `break` give 0 3 9 12 (24), a sieve's marking 14 8 4 2 (28), and an
# empty range none: 67.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];

MAIN:
    let $key := "KAYO_NEVER_SET";
    var $z: i64 = 1;
    no_check let $set := getenv(lend $key);
    if ptr.is_null($set):
        change $z to 0;
    ;if
    var $t: i64 = 0;
    for $i: i64 [from: $z + 9223372036854775800, to: 9223372036854775807, by: $z + 3]:
        change $t to $t + 1;
    ;for
    for $i: i64 [from: $z - 9223372036854775800, to: -9223372036854775808, by: $z - 4]:
        change $t to $t + 1;
    ;for
    for $i: u64 [from: to.u64($z + 5), to: 0, by: $z - 2]:
        change $t to $t + to.i64($i);
    ;for
    for $i: i64 [from: $z, to: $z + 20, by: $z + 3]:
        if $i == $z + 6:
            continue;
        ;if
        if $i >= $z + 15:
            break;
        ;if
        change $t to $t + $i;
    ;for
    for $p: i64 [from: 2, to: $z + 10, by: 1]:
        for $m: i64 [from: $p x $p, to: $z + 30, by: $p]:
            change $t to $t + 1;
        ;for
    ;for
    for $i: i64 [from: $z + 5, to: $z + 4, by: $z + 1]:
        change $t to $t + 100;
    ;for
    no_check my.exit(to.i32($t));
;MAIN
