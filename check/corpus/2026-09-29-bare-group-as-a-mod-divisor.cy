# Finding 2026-09-29 (check run, seed 4698, cut down by hand): ((221 x 2) mod 5) + 1 is bare numbers
# alone, so it is exact (dc83c05): 3. As the divisor of 'p' mod … with 'p' a u8, CCA types it
# at u8 and refuses `x` ("the answer is 442"). As an operand of + it is fine. The oracle runs it.
MAIN {
    let 'p': u8 = 7;
    let 'v': u8 = 'p' mod.floor (((221 times 2) mod.floor 5) plus 1);
}
