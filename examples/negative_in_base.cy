# A negative whole number in a base is the number, its sign and its digits, as in base 10 (owner, 2026-10-24: "(b)",
# over two's complement): `{:b}` of -5 is `-101`, the width counting the sign and `0` padding after it. The stored
# bits are asked for by name: `to.u8.bits($b)` is `11111011`.
MAIN {
    let $a: i64 = -56710;
    let $b: i8 = -5;
    std::print.format("{:X} {:05X} {:x} {:o} {:b} {:b} {:+b} {:08b}\n", $a, $a, $a, $a, $a, $b, 5, $b);
    std::print.format("{:b}\n", to.u8.bits($b));
}
