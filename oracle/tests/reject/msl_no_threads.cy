!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [1, 2, 3, 4];
    let $k: integer.signed.32_bit = 2;
    !no_check inline.%MSL with [lend.mutable $xs -> buffer(0), copy $k -> buffer(1)]:
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    end of inline.%MSL
    std::print.shortest($xs, newline);
!end of MAIN
