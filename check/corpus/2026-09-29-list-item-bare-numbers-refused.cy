# Finding 2026-09-29 (by hand, beside seed 197): CCA refuses a bare-number list item past i64
# ("this is what the program does every time it reaches here"); bare numbers alone are exact
# (dc83c05), so it is 9692729768110883997, which fits u64. The oracle runs it.
MAIN {
    let 'xs': list[u64] = [9692729768110883998 minus 1,];
}
