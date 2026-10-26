# A bin never holds an infinity (owner, 2026-09-29: a): doubling a value
# the program only learns at run time goes past the largest f64 on the
# 1024th step, and the check there stops it. Before, it became infinity
# and the loop ran on.
%extern.C fn exit [$code: i32];
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
fn start [] -> f64 {
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p) { return 1.0; }
    return 0.5;
}
MAIN {
    let mut $x: f64 = start();
    for $i: i32 [from: 1, to: 2000, by: 1] {
        change $x = $x times 2.0;
    }
    no_check my.exit(7);
}
