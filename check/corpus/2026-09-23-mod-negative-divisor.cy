# Finding 2026-09-23 (generator seed 2 with inputs, shrunk): mod by a negative
# divisor gave -1 in CCA (76e5339). design/operators.md: the result has the
# divisor's sign; 1 mod (-3) is -2. Printed as an exit-free comparison.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'x': i64 = 1;
    let 'd': i64 = 'x' mod.floor (-3);
    let 'ok' := "-2 ok";
    if 'd' equal_to -2 { no_check write[1, lend 'ok', 5]; }
    let 'e': i64 = 7 mod.floor (-3);
    if 'e' equal_to -2 { no_check write[1, lend 'ok', 5]; }
}
