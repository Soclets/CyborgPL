# Metal on Apple's GPU, written inline (owner, 2026-10-01, 2026-10-22; built 2026-10-24): `threads` says how many copies
# of the kernel run, a list binds with `lend.mut … -> buffer(0)` (changed when the block ends) and a single value with
# `copy … -> buffer(1)`, a Metal `constant`. A `no_check` site; the oracle checks it and stops, kayo runs it.
MAIN {
    let $xs: std::list[i32] = [1, 2, 3, 4];
    let $k: i32 = 2;
    no_check inline.MSL threads std::list.length(lend $xs) {
        kernel void twice(device int *xs [[buffer(0)]], constant int &k [[buffer(1)]], uint id [[thread_position_in_grid]]) {
            xs[id] = xs[id] * k;
        }
    } with [lend $xs -> buffer(0), copy $k -> buffer(1)];
    std::print.useful($xs, newln);
}
