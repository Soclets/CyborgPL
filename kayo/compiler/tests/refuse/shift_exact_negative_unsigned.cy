# refuse: the value can be outside what a `u32` holds
# An exact answer shifted lands in the width first: -1 shifted at a u32
# width is refused, where its count of 0 alone had been taken as a proof
# that failed at run time (check claude, seed 28123).
extern.C fn exit [$code: i32];
fn one [] -> u32 { return 1; }
MAIN {
    let $v30: u32 = one();
    let $r: u32 = ((0 minus $v30) shr 0) mod.floor 1;
    no_check my.exit(to.i32($r));
}
