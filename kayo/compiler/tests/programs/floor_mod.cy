# `mod` takes the divisor's sign (operators.md): 1 mod -3 is -2.
extern.C fn exit ['code': i32];
extern.C fn labs ['v': i64] -> i64;
MAIN {
    no_check let 'x': i64 = labs[1];
    let 'a': i64 = 1 mod.floor (-3);
    let 'b': i64 = 7 mod.floor (-3);
    let 'd': i64 = 'x' mod.floor (-3);
    let 'e': i64 = 1 mod.floor (-2982098350919537422);
    let 'g': i32 = 1 mod.floor (-2000000000);
    let 'c': i64 = -7 mod.floor 3;
    let 'ok': i32 = 0;
    if 'a' equal_to -2 and 'b' equal_to -2 and 'd' equal_to -2 and 'e' equal_to -2982098350919537421 and 'g' equal_to -1999999999 and 'c' equal_to 2 { no_check my.exit[42]; }
    no_check my.exit[1];
}
