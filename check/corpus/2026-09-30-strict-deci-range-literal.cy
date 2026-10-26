# Finding 2026-09-30 (check run, seed 8049, cut down by hand): in a #strict file, a deci range type
# holding a literal inside it is refused: "the value can be outside what a d64[0.0, 1.0] holds".
# 0.79 is plainly inside [0.0, 1.0]. The oracle runs it.
fn f [] -> i32 {
    let immut 'd': %d64[min: 0.0, max: 1.0] = 0.79;
    return 0;
}
MAIN {
    let immut 'x': i32 = f();
}
