# A strict file's MAIN says its own reason too (check claude, seed 6430):
# 'v' is whatever `pick` declares, any `u32`, and `'v' !== 0 and.then
# (15 mod 'v')` is guarded by the `!==` alone. A range could not lose a
# value it holds at one end; now 0 at the bottom of a `u32` goes.
# 15 mod 1 is 0; 0 !== 3, so 5.
%extern.C fn exit [$code: i32];
fn pick [$n: u32] -> u32 {
    return $n;
}
MAIN {
    let immut $v: u32 = pick(1);
    let mut $code: i32 = 0;
    if $v not_equal 0 and.then (15 mod.floor $v) not_equal 3 {
        change $code = 5;
    }
    no_check my.exit($code);
}
