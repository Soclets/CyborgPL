# A group of bare numbers beside a wrapping operator's typed side is plain
# maths, as wide as its own values need, and wraps to the operator's width
# (check claude, seed 34247): 5 + 2^63, 5 + (2^64 + 1) = 6, 5 + (-1) = 4,
# and (-1) + 5 = 4: 13 + 60 + 400 + 4000 = 4473, exit 121 (the oracle's).
extern.C fn exit ['code': int32];
fn five [] -> uint64 { return 5; }
MAIN {
    let 'v': uint64 = five[];
    let 'a' := 'v' plus.wrap (9223372036854775807 plus 1);
    let 'b' := 'v' plus.wrap (18446744073709551615 plus 2);
    let 'c' := 'v' plus.wrap (0 minus 1);
    let 'd' := (0 minus 1) plus.wrap 'v';
    no_check my.exit[to.int32[('a' mod.floor 100) plus 'b' times 10 plus 'c' times 100 plus 'd' times 1000]];
}
