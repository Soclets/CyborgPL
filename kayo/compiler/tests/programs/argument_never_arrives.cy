# An argument that cannot pass its own narrowing never reaches the callee
# (check claude, 2026-09-29, seed 1027): the -1 stops at the call, which
# traps when it runs, and Precog no longer walks it into `half`, where it
# made the division look wrong every time.
extern.C fn exit [$code: i32];
fn half [$n: u64] -> u64 { return ($n div.floor 10) plus 1; }
MAIN {
    let mut $k: i32 = 2;
    let mut $total: u64 = 0;
    while $k > 0 {
        change $k = $k minus 1;
        if $k equal_to 0 { change $total = half(0 minus 1); }
    }
    let $c: i32 = to.i32($total);
    no_check my.exit($c);
}
