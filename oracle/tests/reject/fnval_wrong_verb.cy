fn look [lent 'a': str.utf8] -> i32 { return 1; }
fn apply ['f': fn[str.utf8] -> i32] -> i32 { return 'f'["x"]; }
MAIN {
    let 'x' := apply[look];
}
