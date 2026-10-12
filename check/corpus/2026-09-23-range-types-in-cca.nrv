# Coverage 2026-09-23: range types (design/input.md) on a name, a parameter
# and a return. CCA (c845110) does not accept them yet.
fn half ['x': int32[min 0, max 1000]] -> int32[min 0, max 500] {
    return 'x' div.floor 2;
}
MAIN {
    let 'p': int32[min 1] = 1;
    let 's': uint16[max 112] = 102;
    let 'h': int32[min 0, max 500] = half[40];
}
