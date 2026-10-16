# Text into a bin (owner, 2026-09-24: b): a sign, digits, a fraction, an
# exponent — nothing else, so inf and nan are not numbers — rounded once
# from the digits to the width; past the largest fails out_of_range, and
# so does a value outside a range type. Which text is read is known only
# at run time: whether an unset variable is there.
extern.C fn exit ['code': int32];
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
fn b64 ['t': str.utf8] -> bin64 {
    let 'v' := to.bin64['t'] otherwise { not_a_number { return -1.0; } out_of_range { return -2.0; } };
    return 'v';
}
fn b32 ['t': str.utf8] -> bin32 {
    let 'v' := to.bin32['t'] otherwise { not_a_number { return -1.0; } out_of_range { return -2.0; } };
    return 'v';
}
fn b16 ['t': str.utf8] -> bin16 {
    let 'v' := to.bin16['t'] otherwise { not_a_number { return -1.0; } out_of_range { return -2.0; } };
    return 'v';
}
fn unit ['t': str.utf8] -> bin64 {
    let 'v' := to.bin64[min 0.0, max 1.0]['t'] otherwise { not_a_number { return -1.0; } out_of_range { return -2.0; } };
    return 'v';
}
MAIN {
    let 'k' := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let 'there' := ptr.is_null[getenv[lend 'k']];
    let mut 's' := str.empty;
    if 'there' { change 's' = "5"; }
    let mut 'got': int32 = 0;
    if b64["2.5e3"] equal_to 2500.0 and b64["-1E-2"] equal_to -0.01 and b64["+" concat 's'] equal_to 5.0 { change 'got' = 'got' plus 1; }
    if b64["nan"] equal_to -1.0 and b64["inf"] equal_to -1.0 and b64["." concat 's'] equal_to -1.0 and b64['s' concat "."] equal_to -1.0 and b64["1_0"] equal_to -1.0 and b64[" " concat 's'] equal_to -1.0 { change 'got' = 'got' plus 2; }
    if b64["1e400"] equal_to -2.0 and b32["1e39"] equal_to -2.0 and b16["6552" concat 's' concat ".0"] equal_to -2.0 { change 'got' = 'got' plus 4; }
    if b16["65519.99999999999999999"] equal_to 65504.0 and b32["340282356779733661637539395458142568447.0"] equal_to 340282346638528859811704183484516925440.0 { change 'got' = 'got' plus 8; }
    if b32["0.1"] equal_to 0.100000001490116119384765625 and b32["1.00000005960464477539062500001"] equal_to 1.00000011920928955078125 { change 'got' = 'got' plus 16; }
    if unit["0." concat 's'] equal_to 0.5 and unit["1." concat 's'] equal_to -2.0 and unit["x"] equal_to -1.0 { change 'got' = 'got' plus 32; }
    if b64["1e-400"] equal_to 0.0 and b16["0.0000000298023223876953125"] equal_to 0.0 and b16["0.0000000298023223876953126"] > 0.0 { change 'got' = 'got' plus 64; }
    no_check my.exit['got'];
}
