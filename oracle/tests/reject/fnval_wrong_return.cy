fn one ['a': int32] -> bool { return true; }
fn apply ['f': fn[int32] -> int32] -> int32 { return 'f'[1]; }
MAIN {
    let 'x' := apply[one];
}
