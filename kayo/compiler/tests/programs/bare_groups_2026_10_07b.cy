# Check claude's findings (corpus 9757dbb9): bare numbers on both sides of a
# comparison are worked exactly, in every build (seed 28563: the product
# 23955622256324454720 was cut to 65 bits under --assert-proofs); a bare
# `pow` group in a list literal too (seed 28093: 454, mod 256 198). The
# comparison is false, so 198 is the answer.
%extern.C proc exit [let $code: i32];
MAIN:
    let $xs: std::ls[u64] := [((16874889984015390149 std::pow 8) mod.floor 665) + 333,];
    var $n: i32 = to.i32($xs[pos: 1] mod.floor 256);
    if 0 >= (-5988905564081113680) x (-4):
        change $n to 0;
    ;if
    no_check my.exit($n);
;MAIN
