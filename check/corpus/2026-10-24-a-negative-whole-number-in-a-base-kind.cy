# Finding 2026-10-24 (generator, the format episode, seed 100): a negative whole number shaped with a base
# kind (`x X o b`). For -56710 as an i64, `{:05X}`: the oracle prints its 64-bit two's complement,
# "FFFFFFFFFFFF227A"; kayo a minus sign and the magnitude, "-DD86" (the width 5 counting the sign). Which
# is meant is not written (design/strings.md, examples/print_format.cy show only positives); asked of
# design claude.
MAIN {
    let $a: i64 = -56710;
    let $b: i8 = -5;
    std::print.format("{:X} {:05X} {:x} {:o} {:b} {:b}\n", $a, $a, $a, $a, $a, $b);
}
