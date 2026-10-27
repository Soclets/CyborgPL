!MAIN:
    var $xs: std::ls[int.sign.32] = [1, 2, 3, 4];
    let $k: int.sign.32 := 2;
    !no_check inline.%MSL threads std::ls.len(lend $xs) with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)]:
        void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    end of inline.%MSL
    std::print.shortest($xs, newln);
!end of MAIN
