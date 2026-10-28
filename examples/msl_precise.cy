# `inline.MSL.precise.wrap` (owner, 2026-10-25): Metal's safe math mode with precise functions and no fused multiply-add,
# and signed overflow wrapping; without them a block builds with Apple's defaults.
MAIN:
    var $xs: std::ls[i32] = [1, 2, 3, 4];
    let $k: i32 := 2;
    no_check inline.%MSL.precise.wrap threads std::ls.len(lend $xs) with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)]:
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    ;inline.%MSL
    std::println($xs);
;MAIN
