# Plain math (plus minus times pow) is exact, and checked where the value lands: a typed declaration,
# a `change`, an argument, a return (owner, 2026-09-23). In between, the one limit is
# the machine's 64 bits. Wrapping (plus.wrap minus.wrap times.wrap pow.wrap) wraps at the size of its values.
# tests/reject/overflow_lands.cy and overflow_past_64_bits.cy are the two that fail.
MAIN {
    let $small: i16 = 5;
    let $big: i64 = (20000 + 20000) + $small;    # 40005: the true answer, and it fits i64
    let $s: i16 = 30000;
    let $twice: i64 = $s + $s;                   # 60000, though no i16 holds it
    let $tiny: i8 = (100 + 100) - 150;             # 50: the 200 in between is fine
    let $v: i8 = 112;
    std::print.useful($big, " ", $twice, " ", $tiny, " ", $v > ($v + $v), newln);   # 224 compared as 224
    let $u: u8 = 5;
    let $below := $u - 10;                          # -5: `:=` takes a type that holds it
    let $x: u32 = 4000000000;
    let $h: u64 = $x plus.wrap $x;                     # wraps at u32, 'x''s size: 3705032704
    let $a: i8 = 100;
    let $b: i16 = 1000;
    let $w := to.i16($a + $b) plus.wrap 1;            # a mix of sizes wraps where you say: 1101
    std::print.useful($below, " ", $h, " ", $w, newln);
}
