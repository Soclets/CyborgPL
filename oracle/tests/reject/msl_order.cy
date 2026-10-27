# `inline.MSL.wrap.precise` (owner, 2026-10-25): Metal's safe math mode with precise functions and no fused multiply-add,
# and signed overflow wrapping; without them a block builds with Apple's defaults.
!MAIN:
    var $xs: std::ls[int.sign.32] = [1, 2, 3, 4];
    let $k: int.sign.32 = 2;
    !no_check inline.%MSL.wrap.precise threads std::ls.len(lend $xs) with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)]:
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    end of inline.%MSL
    std::print.shortest($xs, newln);
!end of MAIN
