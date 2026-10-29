# `inline.MSL.wrap` (owner, 2026-10-25): `-fwrapv`, so a kernel's signed sum past `int`'s top wraps, as it does
# everywhere in two's complement. 1 + 2147483647 wraps to -2147483648; -2147483648 + 2147483647 is -1: exit 1.
MAIN:
    var $xs: std::ls[i32] = [1, -2147483648];
    no_check inline.%MSL.wrap threads 2 with [lend.mut $xs -> buffer(0)]:
        kernel void add(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] + 2147483647;
        }
    ;inline.%MSL
    if $xs[pos: 1] == -2147483648 and $xs[pos: 2] == -1:
        std::exit(code: 1);
    ;if
    std::exit(code: 2);
;MAIN
