# `and.bit.wrap`, `or.bit.wrap`, `xor.wrap` take an operand as its bits (owner,
# 2026-10-07): keep 5, none 0, flip 4294967295 (mod 7: 3), all 4294967295
# (mod 3: 0); 5 x 10 + 0 + 3 + 0 = 53, as the oracle has it.
extern.C fn exit [$code: i32];
fn five [] -> u32 { return 5; }
MAIN {
    let $v: u32 = five();
    let $keep: u32 = $v and.bit.wrap (0 minus 1);
    let $none: u32 = $v and.bit.wrap (1 minus 7);
    let $flip: u32 = $v xor.wrap (1 minus 7);
    let $all: u32 = $v or.bit.wrap (0 minus 1);
    let $a: i32 = to.i32($keep) times 10 plus to.i32($none);
    no_check my.exit(($a plus to.i32($flip mod.floor 7)) plus to.i32($all mod.floor 3));
}
