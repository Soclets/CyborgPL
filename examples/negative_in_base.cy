# A negative whole number in a base is the number, its sign and its digits, as in base 10 (owner, 2026-10-24: "(b)",
# over two's complement): `{:b}` of -5 is `-101`, the width counting the sign and `0` padding after it. The stored
# bits are asked for by name: `to.u8.bits($b)` is `11111011`.
MAIN:
    let $a: i64 := -56710;
    let $b: i8 := -5;
    std::println.format("{base: hex.upper} {base: hex.upper, zeros, width: 5 cols} {base: hex} {base: oct} {base: bin} {base: bin} {base: bin, sign} {base: bin, zeros, width: 8 cols}", $a, $a, $a, $a, $a, $b, 5, $b);
    std::println.format("{base: bin}", to.u8.bits($b));
;MAIN
