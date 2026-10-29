# `div.trunc` and `mod.trunc` worked out at run time, over every i8 and
# five divisors: q x d + r is the dividend, the remainder is below the
# divisor's size and on the dividend's side of zero. The lowest i8 by -1
# is past the type for `div.trunc` and is left out; its `mod.trunc` is 0.
# Lane by lane on a vector too, and C's `fmod` on bins: 4 + 8 + 16 = 28
# when all hold, minus 1 for the first that does not.
%extern.C proc exit [$code: i32];
MAIN:
    let $ds: std::ls[i8] := [-3, -1, 1, 2, 7];
    var $bad: i32 = 0;
    for $d in lend $ds:
        for $i [from: -128, to: 127, by: 1]:
            let $n: i8 := to.i8($i);
            let $r: i8 := $n mod.trunc $d;
            if ($n != -128) or ($d != -1):
                let $q: i8 := $n div.trunc $d;
                if (to.i32($q) x to.i32($d)) + to.i32($r) != $i:
                    change $bad to 1;
                ;if
            ;if
            if ($r < 0) and ($n > 0):
                change $bad to 1;
            ;if
            if ($r > 0) and ($n < 0):
                change $bad to 1;
            ;if
        ;for
    ;for
    var $code: i32 = 4;
    if $bad != 0:
        change $code to 3;
    ;if
    let $xs: std::ls[i32] := [-7, 7, -8, 9];
    let $v: simd[i32, count: 4] := simd.load(lend $xs, pos: 1);
    let $two: simd[i32, count: 4] := simd.fill(value: -2);
    let $q := $v div.trunc $two;
    let $r := $v mod.trunc $two;
    if ($q[pos: 1] == 3) and ($q[pos: 2] == -3) and ($q[pos: 3] == 4) and ($q[pos: 4] == -4) and ($r[pos: 1] == -1) and ($r[pos: 2] == 1) and ($r[pos: 3] == 0) and ($r[pos: 4] == 1):
        change $code to $code + 8;
    ;if
    let $fs: std::ls[f64] := [-370.0, 370.0, 7.5];
    var $fsum: f64 = 0.0;
    for $f in lend $fs:
        change $fsum to $fsum + ($f mod.trunc 360.0);
    ;for
    if $fsum == 7.5:
        change $code to $code + 16;
    ;if
    no_check my.exit($code);
;MAIN
