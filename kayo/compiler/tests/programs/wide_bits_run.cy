# Bitwise operators and shifts on integers past 128 bits (owner, 2026-10-03:
# "Do the bit shit"), worked out at run time at the type's true width, and
# a u128's shl 127 and not.bit at all 128 bits; each answer's digits
# against examples/wide_bits.out, checked against Python. All nine: 9.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    var $right: i32 = 0;
    let $s := "x";
    no_check let $n0: u64 := strlen(lend $s);
    let $z: i32 := to.i32($n0) - 1;
    let $k55: i32 := 55 + $z;
    let $k190: i32 := 190 + $z;
    let $k3: i32 := 3 + $z;
    let $k7: i32 := 7 + $z;
    let $k127: i32 := 127 + $z;
    let $a: %not_std.u256 := 2 std::pow (200 + $z) + 12345;
    let $b: %not_std.u256 := (2 std::pow (255 + $z)) - 1;
    if to.stored.str.utf8($a and.bit $b) == "1606938044258990275541962092341162602522202993782792835313721":
        change $right to $right + 1;
    ;if
    if to.stored.str.utf8($a xor $b) == "57896044618658096104847448245353678384672899991657679497525798221163729506246":
        change $right to $right + 1;
    ;if
    if to.stored.str.utf8($a shl $k55) == "57896044618658097711785492504343953926634992332820282020173567503155675004928":
        change $right to $right + 1;
    ;if
    if to.stored.str.utf8($a shr $k190) == "1024":
        change $right to $right + 1;
    ;if
    let $n: %not_std.i256 := -(2 std::pow (200 + $z));
    if to.stored.str.utf8($n or.bit to.not_std.i256($k7)) == "-1606938044258990275541962092341162602522202993782792835301369":
        change $right to $right + 1;
    ;if
    if to.stored.str.utf8($n shr $k3) == "-200867255532373784442745261542645325315275374222849104412672":
        change $right to $right + 1;
    ;if
    let $zz: %not_std.u256 := to.not_std.u256($z);
    let $all: %not_std.u256 := not.bit $zz;
    if to.stored.str.utf8($all) == "115792089237316195423570985008687907853269984665640564039457584007913129639935":
        change $right to $right + 1;
    ;if
    let $u: %u128 := to.%u128(1 + $z);
    let $top: %u128 := $u shl $k127;
    let $flip: %u128 := not.bit $u;
    if to.stored.str.utf8($top) == "170141183460469231731687303715884105728":
        change $right to $right + 1;
    ;if
    if to.stored.str.utf8($flip) == "340282366920938463463374607431768211454":
        change $right to $right + 1;
    ;if
    no_check my.exit($right);
;MAIN
