!MAIN:
    var $xs: std::ls[int.sign.32] := [1];
    !no_check inline.%MSL.fast threads 1 with [lend.mut $xs -> buffer(0)]:
        kernel void k(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) { xs[id] = 1; }
    end of inline.%MSL
!end of MAIN
