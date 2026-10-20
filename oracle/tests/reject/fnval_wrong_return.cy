fn one ['a': i32] -> bool { return true; }
fn apply ['f': fn[i32] -> i32] -> i32 { return 'f'[1]; }
MAIN {
    let 'x' := apply[one];
}
