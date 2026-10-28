# Bitwise operators and shifts on integers past 128 bits (owner, 2026-10-03: "Do the bit shit"),
# at the type's true width: and i128/u128 shifts and `not.bit` at all 128 bits.
MAIN:
    let $a: %not_std.u256 := 2 ^ 200 + 12345;
    let $b: %not_std.u256 := (2 ^ 255) - 1;
    std::println(($a and.bit $b));
    std::println(($a xor $b));
    std::println(($a shl 55), " ", ($a shr 190));
    let $n: %not_std.i256 := -(2 ^ 200);
    std::println(($n or.bit 7), " ", ($n shr 3));
    let $z: %not_std.u256 := 0;
    let $all: %not_std.u256 := not.bit $z;
    std::println($all);
    let $u: %u128 := 1;
    let $top: %u128 := $u shl 127;
    let $flip: %u128 := not.bit $u;
    std::println($top, " ", $flip);
;MAIN
