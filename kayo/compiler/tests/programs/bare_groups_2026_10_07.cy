# Check claude's findings of 2026-10-07 (seeds 27222, 27256, 27536): a group
# of bare numbers past i64 is worked exactly as a shift count, beside an
# exact answer in `xor`, and in a comparison; a shift waiting for its width
# takes it through `plus.wrap`. r2 is 10803112279996704310 (mod 200: 110),
# the comparison holds (1), r1 and w are 0: 111.
%extern.C proc exit [$code: i32];
fn one [] -> u64:
    ret 1;
;fn one
fn zero [] -> u64:
    ret 0;
;fn zero
fn five [] -> u32:
    ret 5;
;fn five
MAIN:
    let $v2: u64 := zero();
    let $r1: u64 := ($v2 mod.floor 1) shl (1 div.floor 17822710905774487822);
    let $v: u64 := one();
    let $r2: u64 := (to.u64($v) + 7) xor (2582910339 x 4182534762);
    let $u: u32 := five();
    let $w: i32 := (0 shr 0) plus.wrap 0;
    var $n: i32 = to.i32($r2 mod.floor 200) + to.i32($r1) + $w;
    if $u != 4207507906 std::pow 17:
        change $n to $n + 1;
    ;if
    no_check my.exit($n);
;MAIN
