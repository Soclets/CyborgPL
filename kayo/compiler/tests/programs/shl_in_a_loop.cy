# A `shl` whose answer leaves its type is stopped (check claude, seed
# 9148). Layer 3 asked only whether the count was 0 to 31 — it is, 5 down
# to 0 — never whether 2030534677 shifted still fits a u32, "proved"
# the shift, and the binary printed wrapped values. It stops at the first.
%extern.C proc exit [let $code: i32];
fn f1 [let $p1: i64] -> i32:
    var $w10: i8 = 6;
    var $sum: u64 = 0;
    loop:
        if $w10 == 0:
            break;
        ;if
        change $w10 to $w10 - 1;
        let $v: u32 := to.u32($p1 mod.floor 4294967296) shl to.u32($w10);
        change $sum to $sum + to.u64($v);
    ;loop
    ret to.i32($sum mod.floor 100);
;fn f1
MAIN:
    let $x: i32 := f1(2030534677);
    no_check my.exit($x);
;MAIN
