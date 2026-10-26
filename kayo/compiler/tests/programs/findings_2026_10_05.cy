# Four of check claude's findings (2026-10-05, corpus b3bec41, e904fb0), each
# one bit when it gives the oracle's answer: a negated exact answer keeps its
# operands' width for a shift ((-(20 - 1)) shr 1 is -10 at i8); a bare
# number shifted by an exact count takes the count's operands' width (7 shr 4
# at u32 is 0); bare numbers are worked out exactly however big they get
# on the way ((4294967294 pow 12) mod 2147483649 is 16777216); and a deci
# past a narrower one's largest takes the `otherwise` (-180). All four: 15.
%extern.C fn exit [$code: i32];
MAIN {
    let mut $code: i32 = 0;
    let $a: i8 = 20;
    let $b: i8 = (-($a minus 1)) shr 1;
    if $b equal_to -10 { change $code = $code plus 1; }
    let $u: u32 = 5;
    let $c: u32 = 7 shr (($u minus 1) mod.floor 32);
    if $c equal_to 0 { change $code = $code plus 2; }
    let $n: u64 = (4294967294 pow 12) mod.floor 2147483649;
    if $n equal_to 16777216 { change $code = $code plus 4; }
    let $big := to.%d128("1e400") otherwise -21.363;
    let $small := to.%d32($big) otherwise -180.0;
    if $small equal_to -180.0 { change $code = $code plus 8; }
    no_check my.exit($code);
}
