MAIN {
    let mut $xs: std::list[i32] = [1, 2, 3, 4];
    let $k: i32 = 2;
    no_check inline.MSL threads std::list.length(lend $xs) {
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    } with [lend.mut $xs -> buffer(0), copy $xs -> buffer(0)];
    std::print.useful($xs, newln);
}
