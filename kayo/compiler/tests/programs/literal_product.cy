# Two bare numbers multiplied into a u64: an unsigned 64-bit multiply,
# in the ordinary build (folded) and with proofs asserted (computed).
extern.C fn exit ['code': i32];
fn put_u ['n': u64] -> u64 { return 'n'; }
fn f3 ['p1': u32] -> u16 {
    let mut 'w6': i32 = 1;
    let mut 'got': u64 = 0;
    while 'w6' > 0 {
        change 'w6' = 0;
        change 'got' = put_u[2 times 7726793855274730819];
    }
    if 'got' equal_to 15453587710549461638 { return 7; }
    return 0;
}
MAIN {
    no_check my.exit[to.i32[f3[0]]];
}
