# Owner's ruling 2026-09-23 (design/operators.md): a negative shift count is
# a check site. The oracle stops here.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
MAIN {
    let 'v': int8 = 100;
    let 'b': int8 = 'v' shr (-1);
    let 'hi' := "ran on";
    no_check write[1, lend 'hi', 6];
}
