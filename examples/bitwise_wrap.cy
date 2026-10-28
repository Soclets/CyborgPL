# A bitwise operand must fit the operator's width; `.wrap` takes it as its bits instead, as the chip does
# (owner, 2026-10-07: strict by default, `and.bit.wrap`, `or.bit.wrap`, `xor.wrap` the hardware way).
MAIN:
    let $v: u32 := 5;
    let $keep: u32 := $v and.bit.wrap (0 - 1);     # -1's bits are all ones: 'v' unchanged
    let $none: u32 := $v and.bit.wrap (1 - 7);     # -6's bits and 5's: 0
    let $flip: u32 := $v xor.wrap (1 - 7);         # 4294967295
    std::println($keep, " ", $none, " ", $flip);
;MAIN
