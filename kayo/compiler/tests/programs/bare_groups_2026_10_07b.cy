# Check claude's findings (corpus 9757dbb9): bare numbers on both sides of a
# comparison are worked exactly, in every build (seed 28563: the product
# 23955622256324454720 was cut to 65 bits under --assert-proofs); a bare
# `pow` group in a list literal too (seed 28093: 454, mod 256 198). The
# comparison is false, so 198 is the answer.
extern.C fn exit [$code: i32];
MAIN {
    let $xs: std::list[u64] = [((16874889984015390149 pow 8) mod.floor 665) plus 333,];
    let mut $n: i32 = to.i32($xs[pos: 1] mod.floor 256);
    if 0 at_least (-5988905564081113680) times (-4) { change $n = 0; }
    no_check my.exit($n);
}
