# deci's leftovers (owner, 2026-09-29): mod.floor takes the divisor's sign, as integer mod.floor; pow takes a whole
# exponent and rounds like the other operations; a wider deci into a narrower one fails
# too_many_digits; a deci may be a range type, with bounds written as bins' are.
MAIN {
    let 't': d64 = 7.50;
    let 'm': d64 = -7.50;
    std::print.stored['t' mod.floor 2, " ", 'm' mod.floor 2, " ", 't' mod.floor -2, newln];
    let 'r': d64 = 1.05;
    std::print.stored['r' xx 10, " ", 'r' xx 0, " ", 'r' xx -1, newln];
    let 'r2': d128 = 1.05;
    std::print.stored['r2' xx 10, newln];
    let 'n': i32 = 3;
    std::print.stored['r' xx 'n', newln];
    let 'x': d64 = 3.141592653589793;
    let 'y' := to.d32['x'] otherwise 0;
    let 'z': d64 = 2.50;
    let 'zz' := to.d32['z'] otherwise 0;
    std::print.stored['y', " ", 'zz', newln];
    let 'disc': d64[min 0.0, max 1.0] = 0.15;
    let 'p' := to.d64[min 0.0, max 1.0]["1.5"] otherwise 'disc';
    std::print.stored['disc', " ", 'p', newln];
}
