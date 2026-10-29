# Text into a bin (owner, 2026-09-24: b): a sign, digits, a fraction, an
# exponent — nothing else, so inf and nan are not numbers — rounded once
# from the digits to the width; past the largest fails out_of_range, and
# so does a value outside a range type. Which text is read is known only
# at run time: whether an unset variable is there.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
fn b64 [$t: str.utf8] -> f64:
    let $v := to.f64(lend $t)-
    otherwise:
        case not_a_number:
            ret -1.0;
        ;case not_a_number
        case out_of_range:
            ret -2.0;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn b64
fn b32 [$t: str.utf8] -> f32:
    let $v := to.f32(lend $t)-
    otherwise:
        case not_a_number:
            ret -1.0;
        ;case not_a_number
        case out_of_range:
            ret -2.0;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn b32
fn b16 [$t: str.utf8] -> f16:
    let $v := to.f16(lend $t)-
    otherwise:
        case not_a_number:
            ret -1.0;
        ;case not_a_number
        case out_of_range:
            ret -2.0;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn b16
fn unit [$t: str.utf8] -> f64:
    let $v := to.f64[min: 0.0, max: 1.0](lend $t)-
    otherwise:
        case not_a_number:
            ret -1.0;
        ;case not_a_number
        case out_of_range:
            ret -2.0;
        ;case out_of_range
    ;otherwise;
    ret $v;
;fn unit
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $s = str.empty;
    if $there:
        change $s to "5";
    ;if
    var $got: i32 = 0;
    if b64("2.5e3") == 2500.0 and b64("-1E-2") == -0.01 and b64("+" concat $s) == 5.0:
        change $got to $got + 1;
    ;if
    if b64("nan") == -1.0 and b64("inf") == -1.0 and b64("." concat $s) == -1.0 and b64($s concat ".") == -1.0 and b64("1_0") == -1.0 and b64(" " concat $s) == -1.0:
        change $got to $got + 2;
    ;if
    if b64("1e400") == -2.0 and b32("1e39") == -2.0 and b16("6552" concat $s concat ".0") == -2.0:
        change $got to $got + 4;
    ;if
    if b16("65519.99999999999999999") == 65504.0 and b32("340282356779733661637539395458142568447.0") == 340282346638528859811704183484516925440.0:
        change $got to $got + 8;
    ;if
    if b32("0.1") == 0.100000001490116119384765625 and b32("1.00000005960464477539062500001") == 1.00000011920928955078125:
        change $got to $got + 16;
    ;if
    if unit("0." concat $s) == 0.5 and unit("1." concat $s) == -2.0 and unit("x") == -1.0:
        change $got to $got + 32;
    ;if
    if b64("1e-400") == 0.0 and b16("0.0000000298023223876953125") == 0.0 and b16("0.0000000298023223876953126") > 0.0:
        change $got to $got + 64;
    ;if
    no_check my.exit($got);
;MAIN
