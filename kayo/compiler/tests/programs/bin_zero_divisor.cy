# A bin divided by zero is no number (owner, 2026-09-29: a): 0.0 div 0.0
# would be a NaN, and the divisor is asked first. The divisor comes from
# run time, so the check is the program's.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
fn divisor [] -> f64 {
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p) { return 0.0; }
    return 2.0;
}
MAIN {
    let $d := divisor();
    let $q: f64 = 1.0 div.near $d;
    no_check my.exit(7);
}
