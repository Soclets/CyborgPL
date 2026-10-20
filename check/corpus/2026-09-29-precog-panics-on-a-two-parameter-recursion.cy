# Finding 2026-09-29 (by hand, while cutting down seed 3593): cca build panics in Precog
# ("index out of bounds: the len is 1 but the index is 1") on a called recursive function with
# two parameters. With one parameter, or with no call, it builds. The oracle runs it.
fn f ['pr': i32[min: 0, max: 6], 'p1': u64] -> i64 {
    if 'pr' equal_to 0 { return 1; }
    let 'r': i64 = f['pr' minus 1, 'p1'];
    return 'r';
}
MAIN {
    let 'v': i64 = f[3, 587];
}
