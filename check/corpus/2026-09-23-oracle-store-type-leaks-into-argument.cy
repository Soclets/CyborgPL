# Finding 2026-09-23 (generator seed 41 with inputs, shrunk), in the oracle:
# the u8 a call's result is stored into leaks into i64 arithmetic
# inside the call's argument.
fn g ['p': bool] -> u8 { return 0; }
MAIN {
    let 'x': i64 = 0;
    let 'v': u8 = g(0 > (-1) minus 'x');
}
