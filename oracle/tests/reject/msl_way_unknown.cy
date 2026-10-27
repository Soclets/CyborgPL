!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [1];
    !no_check inline.%MSL.fast threads 1 with [lend.mutable $xs -> buffer(0)]:
        kernel void k(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) { xs[id] = 1; }
    end of inline.%MSL
!end of MAIN
