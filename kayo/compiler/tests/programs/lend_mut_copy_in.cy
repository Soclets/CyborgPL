# A plain value stored into a `lend.mut` walk's item is written into the
# list, and borrows nothing: 'v' stays free to use (check claude, seed
# 10411). (5 + 7) x 2 + the first item, 7 = 31; the oracle agrees.
%extern.C proc exit [let $code: i32];
MAIN:
    let $v: u8 := 5;
    var $xs: std::ls[u8] = [1, 2];
    var $t: i32 = 0;
    for $x in lend.mut $xs:
        change $x to $v;
        change $x to 7;
        change $t to $t + to.i32($v) + to.i32($x);
    ;for
    no_check my.exit($t + to.i32($xs[pos: 1]));
;MAIN
