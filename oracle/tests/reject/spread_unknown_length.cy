fn sum [...'nums': i32] -> i32 { return 0; }
MAIN {
    let mut 'zs' := [1, 2];
    list.append[lend.mut 'zs', value: 3];      # no bound: length no longer known here
    let 't' := sum[...'zs'];
}
