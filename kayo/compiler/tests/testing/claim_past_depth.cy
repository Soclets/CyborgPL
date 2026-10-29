%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
fn r [let $d: i32[min: 0, max: 12], let $v: u8, let $w: u16] -> i8:
    if $d >= 12:
        ret to.i8($v);
    ;if
    ret r($d + 1, $v, $w);
;fn r
MAIN:
    let $s := "x";
    no_check let $n: u64 := strlen(lend $s);
    let $w: u16 := to.u16($n mod.floor 65536);
    let $a := r(0, 6, $w);
    let $b := r(12, 4, $w);
    no_check my.exit(to.i32($a + $b));
;MAIN
