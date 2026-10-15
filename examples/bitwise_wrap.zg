# A bitwise operand must fit the operator's width; `.wrap` takes it as its bits instead, as the chip does
# (owner, 2026-10-07: strict by default, `and.bit.wrap`, `or.bit.wrap`, `xor.wrap` the hardware way).
MAIN {
    let 'v': uint32 = 5;
    let 'keep': uint32 = 'v' and.bit.wrap (0 minus 1);     # -1's bits are all ones: 'v' unchanged
    let 'none': uint32 = 'v' and.bit.wrap (1 minus 7);     # -6's bits and 5's: 0
    let 'flip': uint32 = 'v' xor.wrap (1 minus 7);         # 4294967295
    std::print.useful['keep', " ", 'none', " ", 'flip', newln];
}
