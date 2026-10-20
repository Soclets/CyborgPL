# Owner's ruling 2026-09-23 (design/operators.md): a negative shift count is
# a check site. The oracle stops here.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'v': i8 = 100;
    let 'b': i8 = 'v' shr (-1);
    let 'hi' := "ran on";
    no_check write[1, lend 'hi', 6];
}
