# A bin power with no number (owner, 2026-09-29: a): -1.0 xx 0.5 would be
# a NaN, so the power's site stops the program; 2.0 xx 0.5 is fine.
extern.C fn exit [$code: i32];
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
fn base [] -> f64 {
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p) { return -1.0; }
    return 2.0;
}
MAIN {
    let $r: f64 = 2.0 pow 0.5;
    let $b := base();
    let $q: f64 = $b pow 0.5;
    no_check my.exit(7);
}
