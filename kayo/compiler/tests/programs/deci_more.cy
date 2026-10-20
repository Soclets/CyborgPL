# deci's leftovers (owner, 2026-09-29), as examples/deci_more.cpls has
# them, checked by the program itself against the oracle's text: mod takes
# the divisor's sign, xx a whole exponent, a narrower deci fails
# too_many_digits, and a deci may be a range type. One point each: 19.
extern.C fn exit ['code': i32];
fn same ['v': d64, 'want': str.utf8] -> i32 {
    if to.stored.str.utf8['v'] equal_to 'want' { return 1; }
    return 0;
}
fn in_unit ['v': d64] -> i32 {
    let 'p' := to.d64[min 0.0, max 1.0]['v'] otherwise { out_of_range { return 0; } };
    if 'p' at_least 0.0 { return 1; }
    return 2;
}
MAIN {
    let mut 'got': i32 = 0;
    let 't': d64 = 7.50;
    let 'm': d64 = -7.50;
    change 'got' = 'got' plus same['t' mod.floor 2, "1.50"] plus same['m' mod.floor 2, "0.50"] plus same['t' mod.floor -2, "-0.50"];
    let 'c': d64 = -4.0;
    change 'got' = 'got' plus same['c' mod.floor 2, "0.0"] plus same['c' mod.floor -2, "-0.0"] plus same['t' mod.floor 0.7, "0.50"];
    let 'r': d64 = 1.05;
    change 'got' = 'got' plus same['r' pow 10, "1.628894626777441"] plus same['r' pow 0, "1"] plus same['r' pow -1, "0.9523809523809524"];
    let 'r2': d128 = 1.05;
    if to.stored.str.utf8['r2' pow 10] equal_to "1.62889462677744140625" { change 'got' = 'got' plus 1; }
    let 'n': i32 = 3;
    change 'got' = 'got' plus same['r' pow 'n', "1.157625"] plus same['c' pow 3, "-64.000"];
    let 'x': d64 = 3.141592653589793;
    let 's': d32 = 7;
    let 'y' := to.d32['x'] otherwise 's';
    let 'z': d64 = 2.50;
    let 'zz' := to.d32['z'] otherwise 's';
    if to.stored.str.utf8['y'] equal_to "7" and to.stored.str.utf8['zz'] equal_to "2.50" { change 'got' = 'got' plus 1; }
    let 'disc': d64[min 0.0, max 1.0] = 0.15;
    let 'p' := to.d64[min 0.0, max 1.0]["1.5"] otherwise 'disc';
    change 'got' = 'got' plus same['disc', "0.15"] plus same['p', "0.15"];
    change 'got' = 'got' plus in_unit[0.25] plus in_unit[1.5];
    let 'q' := to.d64[min 0.0, max 1.0]["0.5"] otherwise 'disc';
    change 'got' = 'got' plus same['q', "0.5"];
    let 'e': i64 = 5;
    let 'f' := to.d32[min 0.0, max 10.0]['e'] otherwise 's';
    if to.stored.str.utf8['f'] equal_to "5" { change 'got' = 'got' plus 1; }
    let 'w': f64 = 5.5;
    let 'g' := to.stored.d64[min 0.0, max 1.0]['w'] otherwise 'disc';
    change 'got' = 'got' plus same['g', "0.15"];
    no_check my.exit['got'];
}
