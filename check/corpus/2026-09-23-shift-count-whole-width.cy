# Owner's ruling 2026-09-23 (design/operators.md): a shift count outside
# 0..width-1 is a check site, for shl and shr alike. The oracle stops here.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'v': i8 = 100;
    let 'a': i8 = 'v' shr 8;
    let 'hi' := "ran on";
    no_check write(1, lend 'hi', 6);
}
