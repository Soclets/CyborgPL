# Finding 2026-09-23 (generator seed 41 with inputs, shrunk), in the oracle:
# the uint8 a call's result is stored into leaks into int64 arithmetic
# inside the call's argument.
fn g ['p': bool] -> uint8 { return 0; }
MAIN {
    let 'x': int64 = 0;
    let 'v': uint8 = g[0 > (-1) minus 'x'];
}
