# `*0.1` is exactly what is written, a deci (owner, 2026-10-02): d64 under
# `:=`, the width of a typed place otherwise. 0.3 x 10 + 19.99 x 100 + 0.5 x 2
# = 3 + 1999 + 1 = 2003, 211 mod 256; the oracle agrees.
%extern.C proc exit [$code: i32];
MAIN:
    let $price := exactly 19.99;
    let $tenth := exactly 0.1;
    let $total := $tenth + exactly 0.2;
    let $half: %d32 := exactly 0.5;
    let $n: i32 := to.i32(($total x 10) + ($price x 100) + to.stored.%d64($half x 2)) ?? 0;
    no_check my.exit($n mod.floor 256);
;MAIN
