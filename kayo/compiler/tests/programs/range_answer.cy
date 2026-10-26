# A call's answer of a range type, checked where it was made: the caller
# tells LLVM its range (owner, 2026-09-29: c), so a loop bounded by it is
# worked out at 64 bits. `pick` gives 3 here (no SYNAPSE_NO_SUCH_VARIABLE),
# and the sum of 1 to 3 is 6.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
fn pick [] -> i64[min: 0, max: 1000] or fail[missing] {
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p) { return 3; }
    fail missing;
}
MAIN {
    let $n := pick() otherwise 0;
    let mut $sum: i64 = 0;
    let mut $i: i64 = 1;
    while $i at_most $n {
        change $sum = $sum plus $i;
        change $i = $i plus 1;
    }
    no_check my.exit(to.i32($sum));
}
