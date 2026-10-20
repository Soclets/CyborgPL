# Finding 2026-09-23 (generator seed 90), in the oracle: a wrapping operator of
# two literals, compared with a u64 name, is refused as having no width.
MAIN {
    let 'n': u64 = 5;
    let 'b': bool = 'n' > (0 minus.wrap 1);
}
