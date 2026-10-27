# A negative whole number in a base is the number, its sign and its digits, as in base 10 (owner, 2026-10-24: "(b)",
# over two's complement): `{:b}` of -5 is `-101`, the width counting the sign and `0` padding after it. The stored
# bits are asked for by name: `to.u8.bits($b)` is `11111011`.
!MAIN:
    let $a: integer.signed.64_bit = negative 56710;
    let $b: integer.signed.8_bit = negative 5;
    std::print.format("{base: hexadecimal.upper} {base: hexadecimal.upper, zeros, width: 5 columns} {base: hexadecimal} {base: octal} {base: binary} {base: binary} {base: binary, sign} {base: binary, zeros, width: 8 columns}{newline}", $a, $a, $a, $a, $a, $b, 5, $b);
    std::print.format("{base: binary}{newline}", to.integer.unsigned.8_bit.bits($b));
!end of MAIN
