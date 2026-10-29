# A `:=` list of bare numbers has no width written, so it holds them all:
# 2^63 is not narrowed into the i64 range (it was refused, wrongly).
%extern.C proc exit [let $code: i32];
MAIN:
    let $xs := [9223372036854775808, 1];
    no_check my.exit(to.i32($xs[pos: 2]));
;MAIN
