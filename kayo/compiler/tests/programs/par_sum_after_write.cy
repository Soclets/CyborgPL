# A parallel sum over a list after a loop that wrote it (check claude,
# 2026-10-19): Precog knew the list's length, folded the sum loop's stop
# to it, and each slice ran from its own first trip to the loop's end,
# 8000 ones summing to 16001. 8000 ones, then 4538 trues of 20000
# flags: 12538, and 12538 mod 256 is 250.
%extern.C proc exit [let $code: i32];
MAIN:
    var $b: std::ls[u8] = std::ls.fill(value: 0, count: 8000);
    for $i: u64 [from: 1, to: 8000, by: 1]:
        change $b[pos: $i] to 1;
    ;for
    var $c: u64 = 0;
    for $i: u64 [from: 1, to: std::ls.len(lend $b), by: 1]:
        change $c to $c + to.u64($b[pos: $i]);
    ;for
    var $f: std::ls[bool] = std::ls.fill(value: false, count: 20000);
    for $i: u64 [from: 1, to: 4538, by: 1]:
        change $f[pos: $i x 4] to true;
    ;for
    for $i: u64 [from: 1, to: std::ls.len(lend $f), by: 1]:
        if $f[pos: $i]:
            change $c to $c + 1;
        ;if
    ;for
    no_check my.exit(to.i32($c mod.floor 256));
;MAIN
