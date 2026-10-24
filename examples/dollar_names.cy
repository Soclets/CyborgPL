# A variable is written `$name`, or quoted, `'total due'`: the same variable either way (owner, 2026-10-22: "Why `$`?
# Even though, it requires a shift, it's easier to write"). Good practice: `$`, and quotes only for a name that needs
# them. A call takes `( )`, a definition keeps `[ ]` (owner, 2026-10-22: "`( )` runs something; `[ ]` describes or
# holds something").
fn twice [$n: i64] -> i64 { return $n x 2; }

MAIN {
    let $total := 1;
    change 'total' = $total + 1;
    let 'total due' := twice($total) x 10;
    let $xs: std::list[i64] = [$total, 'total due'];
    std::print.useful($total, " ", 'total due', " ", $xs[pos: 2], newln);
}
