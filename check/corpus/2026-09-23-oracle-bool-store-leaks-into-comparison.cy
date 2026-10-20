# Finding 2026-09-23 (generator seed 93), in the oracle: the bool a comparison
# is stored into reaches a wrapping operator inside its operand.
MAIN {
    let 'w': u32 = 3;
    let 'b': bool = (5 minus.wrap 'w') > 1;
}
