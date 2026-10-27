# Bitwise operators and shifts on integers past 128 bits (owner, 2026-10-03: "Do the bit shit"),
# at the type's true width: and i128/u128 shifts and `not.bit` at all 128 bits.
!MAIN:
    let $a: %not_std.u256 = 2 power 200 plus 12345;
    let $b: %not_std.u256 = (2 power 255) minus 1;
    std::print.shortest(($a and.bit $b), newline);
    std::print.shortest(($a xor $b), newline);
    std::print.shortest(($a shift_left 55), " ", ($a shift_right 190), newline);
    let $n: %not_std.i256 = negate (2 power 200);
    std::print.shortest(($n or.bit 7), " ", ($n shift_right 3), newline);
    let $z: %not_std.u256 = 0;
    let $all: %not_std.u256 = not.bit $z;
    std::print.shortest($all, newline);
    let $u: integer.unsigned.%128_bit = 1;
    let $top: integer.unsigned.%128_bit = $u shift_left 127;
    let $flip: integer.unsigned.%128_bit = not.bit $u;
    std::print.shortest($top, " ", $flip, newline);
!end of MAIN
