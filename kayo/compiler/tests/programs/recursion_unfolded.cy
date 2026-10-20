# kayo inlines a function that calls itself into itself a few levels
# (owner, 2026-10-17): what each computes stays the same. The order of
# effects (a walk writing before and after its call), a failure climbing
# out from deep down, two calls with different arguments, recursion far
# deeper than the levels unfolded, and two functions calling each other.
extern.C fn exit ['code': i32];

fn fib ['n': i64[min 0, max 92]] -> i64 {
    if 'n' < 2 { return 'n'; }
    return fib['n' - 1] + fib['n' - 2];
}

fn walk ['n': i32[min 0, max 6]] -> str.utf8 {
    if 'n' equal_to 0 { return str.empty; }
    return to.stored.str.utf8['n'] concat walk['n' - 1] concat to.stored.str.utf8[10 + 'n'];
}

fn depth ['n': i32[min 0, max 5000]] -> i32 {
    if 'n' equal_to 0 { return 0; }
    return depth['n' - 1] + 1;
}

fn find ['n': i32[min 0, max 50]] -> i32 or fail[missing] {
    if 'n' equal_to 0 { fail missing; }
    if 'n' equal_to 7 { return 7; }
    let 'r' := find['n' - 1] otherwise fail;
    return 'r' + 1;
}

fn ack ['m': i64[min 0, max 2], 'n': i64[min 0, max 20]] -> i64 {
    if 'm' equal_to 0 { return 'n' + 1; }
    if 'n' equal_to 0 { return ack['m' - 1, 1]; }
    let 'inner' := ack['m', 'n' - 1];
    if 'inner' greater_than 20 { return 0; }
    return ack['m' - 1, 'inner'];
}

fn even ['n': i32[min 0, max 5000]] -> bool {
    if 'n' equal_to 0 { return true; }
    return odd['n' - 1];
}

fn odd ['n': i32[min 0, max 5000]] -> bool {
    if 'n' equal_to 0 { return false; }
    return even['n' - 1];
}

MAIN {
    let mut 'got': i32 = 0;
    if fib[30] equal_to 832040 { change 'got' = 'got' + 1; }
    if walk[4] equal_to "432111121314" { change 'got' = 'got' + 2; }
    if depth[4000] equal_to 4000 { change 'got' = 'got' + 4; }
    let 'a' := find[20] otherwise -1;
    let 'b' := find[5] otherwise -1;
    if 'a' equal_to 20 and 'b' equal_to -1 { change 'got' = 'got' + 8; }
    if ack[2, 3] equal_to 9 { change 'got' = 'got' + 16; }
    if even[4000] and odd[3999] and not even[777] { change 'got' = 'got' + 32; }
    no_check my.exit['got'];
}
