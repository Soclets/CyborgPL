# Integers of any width (owner, 2026-10-03), worked out at run time (the
# values pass through C): a !std.uint256's largest halved and back, 2^255
# doubled less one, wrapping at the type's true width (a u128's largest
# plus.wrap 1 is 0), a !std.int256's floored mod, two int128s added at 128
# bits in between, a !std.uint4 wrapping at 16; and the digits of a wide
# value as text and into a f64. Each right group adds its bit: 255.
extern.C fn exit ['code': i32];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 's' := "x";
    no_check let 'n': u64 = strlen[lend 's'];
    let 'z': i32 = to.i32['n'] minus 1;
    let 'max': not_std.u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;
    let 'half': not_std.u256 = 'max' div.floor (2 plus 'z');
    let 'top': not_std.u256 = 2 pow (255 plus 'z');
    let 'p': not_std.i256 = -(2 pow 200);
    let 'c': i128 = 170141183460469231731687303715884105727;
    let 'avg': i128 = ('c' plus 'c') div.floor (2 plus 'z');
    let 'm': u128 = 340282366920938463463374607431768211455;
    let mut 'code': i32 = 0;
    if 'half' plus 'half' plus 1 equal_to 'max' { change 'code' = 'code' plus 1; }
    if ('top' minus 1 plus 'top') equal_to 'max' { change 'code' = 'code' plus 2; }
    if ('top' plus.wrap 'top') equal_to 0 { change 'code' = 'code' plus 4; }
    if ('p' mod.floor (7 plus 'z')) equal_to 3 { change 'code' = 'code' plus 8; }
    if 'avg' equal_to 'c' { change 'code' = 'code' plus 16; }
    if (('m' plus.wrap 1) equal_to 0) and (('m' times.wrap 'm') equal_to 1) { change 'code' = 'code' plus 32; }
    let 'f': not_std.u4 = to.not_std.u4[9 plus 'z'];
    if ('f' plus.wrap 9) equal_to 2 { change 'code' = 'code' plus 64; }
    let 'digits' := to.stored.str.utf8['max'];
    let 'neg' := to.stored.str.utf8['p' div.near (3 plus 'z')];
    let 'as_bin': f64 = to.f64['top'];
    if ('digits' equal_to "115792089237316195423570985008687907853269984665640564039457584007913129639935") and ('neg' equal_to "-535646014752996758513987364113720867507400997927597611767125") and ('as_bin' equal_to 57896044618658097711785492504343953926634992332820282019728792003956564819968.0) { change 'code' = 'code' plus 128; }
    no_check my.exit['code'];
}
