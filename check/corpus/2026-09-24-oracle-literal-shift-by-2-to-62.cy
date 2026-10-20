# Finding 2026-09-24 (generator seed 1125, shrunk), in the oracle: a shift
# count of 2^62 runs; the owner's shift-count ruling makes it a check site.
MAIN {
    let 'v7': bool = (to.i64[0] shr 4611686018427387904) > 0;
}
