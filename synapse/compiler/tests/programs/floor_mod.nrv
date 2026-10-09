# `mod` takes the divisor's sign (operators.md): 1 mod -3 is -2.
extern.C fn exit ['code': int32];
extern.C fn labs ['v': int64] -> int64;
MAIN {
    no_check let 'x': int64 = labs[1];
    let 'a': int64 = 1 mod.floor (-3);
    let 'b': int64 = 7 mod.floor (-3);
    let 'd': int64 = 'x' mod.floor (-3);
    let 'e': int64 = 1 mod.floor (-2982098350919537422);
    let 'g': int32 = 1 mod.floor (-2000000000);
    let 'c': int64 = -7 mod.floor 3;
    let 'ok': int32 = 0;
    if 'a' equal_to -2 and 'b' equal_to -2 and 'd' equal_to -2 and 'e' equal_to -2982098350919537421 and 'g' equal_to -1999999999 and 'c' equal_to 2 { no_check my.exit[42]; }
    no_check my.exit[1];
}
