# A list grown to 16384 `i32`s (64 KB: doubling makes its capacity whole pages) doubled on the GPU three times
# by one block, so the block's pipeline is made once and kept, and the list may go with no copy when its
# memory is page-aligned (owner, 2026-10-25). Every element is 8 after: exit 8, or how many are not.
MAIN {
    let mut $xs: std::list[i32] = [];
    for $i [from: 1, to: 16384, by: 1] { std::list.append(lend.mut $xs, value: 1); }
    let $k: i32 = 2;
    for $round [from: 1, to: 3, by: 1] {
        no_check inline.MSL threads std::list.length(lend $xs) {
            kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
                xs[id] = xs[id] * k;
            }
        } with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)];
    }
    let mut $wrong: i32 = 0;
    for $x in lend $xs {
        if $x not_equal 8 { change $wrong = $wrong + 1; }
    }
    if $wrong equal_to 0 { std::exit(code: 8); }
    std::exit(code: $wrong);
}
