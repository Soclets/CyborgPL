MAIN {
    let mut $xs: std::list[i32] = [1];
    no_check inline.MSL.fast threads 1 {
        kernel void k(device int *xs [[buffer(0)]], uint id [[thread_position_in_grid]]) { xs[id] = 1; }
    } with [lend.mut $xs -> buffer(0)];
}
