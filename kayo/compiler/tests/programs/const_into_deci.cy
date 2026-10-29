# A const's whole number into a deci is its value (check claude, seed
# 48186): the conversion took the constant's raw bits as a deci, so 1234
# came out as 0.000…0001234 and back as nothing whole. 1234 mod 256 is 210.
%extern.C proc exit [let $code: i32];
const $top: i64 := 1000;
MAIN:
    const $c: i64 := 234;
    let $d := to.%d128($top) + to.%d128($c);
    let $n: i64 := to.i64($d x 1) ?? -1;
    no_check my.exit(to.i32($n mod.floor 256));
;MAIN
