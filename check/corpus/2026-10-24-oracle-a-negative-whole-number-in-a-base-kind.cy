# Finding 2026-10-24 (generator, the format episode, seed 100): a negative whole number shaped with a base
# kind (`x X o b`). Ruled (owner, 2026-10-24; oracle a1dcbd3a): a sign and the magnitude, as in base 10, the
# width counting the sign and `0` padding after it; the stored bits are asked for by name (`to.u8.bits`).
# For -56710 `{:X}` is "-DD86", as kayo printed; the oracle printed the 64-bit two's complement,
# "FFFFFFFFFFFF227A", until a1dcbd3a.
# Shapes in words since 2026-10-25 (owner; oracle 0c746ed3): `{:X}` is `{base: hex.upper}`, `{:05X}` `{base:
# hex.upper, zeros, width: 5 cols}`, `{:+b}` `{sign, base: bin}`.
MAIN {
    let $a: i64 = -56710;
    let $b: i8 = -5;
    std::print.format("{base: hex.upper} {base: hex.upper, zeros, width: 5 cols} {base: hex} {base: oct} {base: bin} {base: bin} {base: bin, zeros, width: 8 cols} {sign, base: bin}\n", $a, $a, $a, $a, $a, $b, $b, 5);
}
