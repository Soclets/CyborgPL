# Parallel `for` (owner, 2026-09-30: "do para for"): loops whose iterations
# are proven apart run in slices on the cores — each writes its own element,
# or adds into one total — and give what the loop in order gives. The sum
# is 0 or more every time, so its one check at the end is the loop's.
# (Their total mod 256, as the oracle computes it: 160.)
%extern.C proc exit [$code: i32];
MAIN:
    let $n: i64 := 300000;
    var $xs: std::ls[i64] = std::ls.fill(value: 3, count: $n);
    for %para $i: i64 [from: 1, to: $n, by: 1]:
        change $xs[pos: $i] to ($xs[pos: $i] x 7) mod.floor 11;
    ;for
    var $total: i64 = 0;
    for %para $x in lend $xs:
        change $total to $total + ($x mod.floor 7);
    ;for
    var $grid: std::ls[i64] = std::ls.fill(value: 0, count: 2000);
    for %para $r: i64 [from: 1, to: 2000, by: 1]:
        var $row: i64 = 0;
        for %para $c: i64 [from: 1, to: 200, by: 1]:
            change $row to $row + (($r x $c) mod.floor 5);
        ;for
        change $grid[pos: $r] to $row;
    ;for
    var $g: i64 = 0;
    for %para $v in lend $grid:
        change $g to $g + $v;
    ;for
    no_check my.exit(to.i32(($total + $g) mod.floor 256));
;MAIN
