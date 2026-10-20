# deci, IEEE 754 decimal (owner, 2026-09-29), as examples/deci.cpls has it
# but checked by the program itself: each value's text against the
# oracle's, one point each, so the exit is how many agree (30).
extern.C fn exit ['code': i32];
fn same ['v': d64, 'want': str.utf8] -> i32 {
    if to.stored.str.utf8['v'] equal_to 'want' { return 1; }
    return 0;
}
MAIN {
    let mut 'got': i32 = 0;
    let 'a': d64 = 0.1;
    let 'b': d64 = 'a' plus 0.2;
    change 'got' = 'got' plus same['b', "0.3"];
    if 'b' equal_to 0.3 { change 'got' = 'got' plus 1; }
    let 'price': d64 = 19.99;
    let 'sub': d64 = 'price' times 3;
    let 'tax': d64 = 'sub' times 0.075;
    change 'got' = 'got' plus same['sub', "59.97"] plus same['tax', "4.49775"] plus same['sub' plus 'tax', "64.46775"];
    let 'h': d64 = 2.50;
    change 'got' = 'got' plus same['h' times 2, "5.00"] plus same['h', "2.50"];
    if 'h' times 2 equal_to 5 { change 'got' = 'got' plus 1; }
    let 'third': d64 = 1;
    change 'got' = 'got' plus same['third' div.near 3, "0.3333333333333333"] plus same['h' div.near 2, "1.25"];
    let 'e1': d32 = 1.000005;
    if to.stored.str.utf8['e1' plus 0.0000005] equal_to "1.000006" { change 'got' = 'got' plus 1; }
    let 'w': f64 = 0.1;
    change 'got' = 'got' plus same[to.stored.d64['w'], "0.1000000000000000"];
    if to.stored.str.utf8[to.stored.d128['w']] equal_to "0.1000000000000000055511151231257827" { change 'got' = 'got' plus 1; }
    if to.stored.str.utf8[to.useful.d128['w']] equal_to "0.1" { change 'got' = 'got' plus 1; }
    let 'n': i32 = 12345;
    change 'got' = 'got' plus same[to.d64['n'], "12345"];
    let 'big': i64 = 12345678901234567;
    let 'bd' := to.d64['big'] otherwise 'price';
    let 'fits': i64 = 1234567890123456;
    let 'fd' := to.d64['fits'] otherwise 'price';
    change 'got' = 'got' plus same['bd', "19.99"] plus same['fd', "1234567890123456"];
    let 't' := to.d64["1.250"] otherwise 'price';
    let 'u' := to.d64["0.12345678901234567"] otherwise { too_many_digits { is 'a'; } not_a_number { is 'h'; } out_of_range { is 'h'; } };
    let 'v' := to.d64["abc"] otherwise { not_a_number { is 'h'; } too_many_digits { is 'a'; } out_of_range { is 'a'; } };
    let 'x' := to.d64["1e999"] otherwise { out_of_range { is 'third'; } too_many_digits { is 'a'; } not_a_number { is 'a'; } };
    change 'got' = 'got' plus same['t', "1.250"] plus same['u', "0.1"] plus same['v', "2.50"] plus same['x', "1"];
    let 'i' := to.i32['h' times 2] otherwise -1;
    let 'j' := to.i32['h'] otherwise -1;
    if 'i' equal_to 5 and 'j' equal_to -1 { change 'got' = 'got' plus 1; }
    if to.f64['h'] equal_to 2.5 { change 'got' = 'got' plus 1; }
    let 'neg': d64 = -1.5;
    change 'got' = 'got' plus same[-'neg', "1.5"] plus same['neg', "-1.5"];
    if 'neg' < 'h' and not ('h' < 'neg') and 'h' at_least 2.5 { change 'got' = 'got' plus 1; }
    let 'g': d64 = 0.1 plus 0.2;
    change 'got' = 'got' plus same['g', "0.3"];
    let 'r': d32 = 2.5;
    let 'wide' := to.d64['r'];
    change 'got' = 'got' plus same['wide' times 'h', "6.250"];
    let 'z': d64 = 0;
    change 'got' = 'got' plus same[-'z', "-0"] plus same['h' minus 'h', "0.00"];
    no_check my.exit['got'];
}
