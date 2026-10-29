# `std::list.page_aligned` and `std::list.not_page_aligned` bound to a GPU block (owner, 2026-10-25): the first
# made page-aligned at every size, a literal grown by an append, the second always copied in and back. Tripled,
# [1..5] sums to 45 and 8192 twos to 49152: exit 45 + 49152 / 8192 = 51 when both are right.
MAIN:
    var $xs: std::ls.page_aligned[i32] = [1, 2, 3, 4];
    std::ls.append(lend.mut $xs, value: 5);
    let $k: i32 := 3;
    no_check inline.%MSL threads std::ls.len(lend $xs) with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)]:
        kernel void times(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    ;inline.%MSL
    var $ys: std::ls.not_page_aligned[i32] = std::ls.fill(value: 2, count: 8192);
    no_check inline.%MSL threads std::ls.len(lend $ys) with [lend.mut $ys -> buffer(0), copy $k -> buffer(1)]:
        kernel void times(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    ;inline.%MSL
    var $sum: i32 = 0;
    for $x in lend $xs:
        change $sum to $sum + $x;
    ;for
    var $twos: i32 = 0;
    for $y in lend $ys:
        change $twos to $twos + $y;
    ;for
    std::exit(code: $sum + $twos div.trunc 8192);
;MAIN
