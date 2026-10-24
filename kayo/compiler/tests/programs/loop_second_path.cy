# Layer 2 ends a loop's trip only when every path through it has come
# round. It used to start the next trip at the first arrival, wiping the
# path still on its way: the call on the `if` side was never analysed, and
# layer 1 folded a value from the other call into both (check claude,
# generator seed 2). A process id is never 0, so 'b' is true.
extern.C fn exit [$code: i32];
extern.C fn getpid [] -> i32;
fn pick [$n: i64] -> i64 { return $n times 10; }
MAIN {
    no_check let $p := getpid();
    let $b: bool = $p > 0;
    let mut $got: i64 = 0;
    let mut $w: i64 = 1;
    while $w > 0 {
        change $w = 0;
        if $b { change $got = pick(3); }
    }
    let $last := pick(1);
    no_check my.exit(to.i32($got plus $last));
}
