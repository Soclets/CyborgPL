# A literal into a f16 or f32 is decided and written from its digits,
# rounded once (oracle 8705014): read at 64 bits first, the first two
# land on the limit, tie to infinity, and were refused.
extern.C fn exit [$code: i32];
MAIN {
    let mut $got: i32 = 0;
    let $a: f32 = 340282356779733661637539395458142568447.0;
    if $a equal_to 340282346638528859811704183484516925440.0 { change $got = $got plus 1; }
    let $b: f16 = 65519.999999999999999999;
    if $b equal_to 65504.0 { change $got = $got plus 2; }
    let $c: f32 = 1.00000005960464477539062500001;
    if $c equal_to 1.00000011920928955078125 { change $got = $got plus 4; }
    let $d: f16 = -65519.9;
    if $d equal_to -65504.0 { change $got = $got plus 8; }
    no_check my.exit($got);
}
