# `std::list.page_aligned` and `std::list.not_page_aligned` bound to a GPU block (owner, 2026-10-25): the first
# made page-aligned at every size, a literal grown by an append, the second always copied in and back. Tripled,
# [1..5] sums to 45 and 8192 twos to 49152: exit 45 + 49152 / 8192 = 51 when both are right.
MAIN {
    let mut $xs: std::list.page_aligned[i32] = [1, 2, 3, 4];
    std::list.append(lend.mut $xs, value: 5);
    let $k: i32 = 3;
    no_check inline.%MSL threads std::list.length(lend $xs) {
        kernel void times(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    } with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)];
    let mut $ys: std::list.not_page_aligned[i32] = std::list.fill(value: 2, count: 8192);
    no_check inline.%MSL threads std::list.length(lend $ys) {
        kernel void times(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    } with [lend.mut $ys -> buffer(0), copy $k -> buffer(1)];
    let mut $sum: i32 = 0;
    for $x in lend $xs { change $sum = $sum + $x; }
    let mut $twos: i32 = 0;
    for $y in lend $ys { change $twos = $twos + $y; }
    std::exit(code: $sum + $twos div.trunc 8192);
}
