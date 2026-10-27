# Plain math (plus minus times pow) is exact, and checked where the value lands: a typed declaration,
# a `change`, an argument, a return (owner, 2026-09-23). In between, the one limit is
# the machine's 64 bits. Wrapping (plus.wrap minus.wrap times.wrap pow.wrap) wraps at the size of its values.
# tests/reject/overflow_lands.cy and overflow_past_64_bits.cy are the two that fail.
!MAIN:
    let $small: integer.signed.16_bit = 5;
    let $big: integer.signed.64_bit = (20000 plus 20000) plus $small;    # 40005: the true answer, and it fits i64
    let $s: integer.signed.16_bit = 30000;
    let $twice: integer.signed.64_bit = $s plus $s;                   # 60000, though no i16 holds it
    let $tiny: integer.signed.8_bit = (100 plus 100) minus 150;             # 50: the 200 in between is fine
    let $v: integer.signed.8_bit = 112;
    std::print.shortest($big, " ", $twice, " ", $tiny, " ", $v greater_than ($v plus $v), newline);   # 224 compared as 224
    let $u: integer.unsigned.8_bit = 5;
    let $below := $u minus 10;                          # -5: `:=` takes a type that holds it
    let $x: integer.unsigned.32_bit = 4000000000;
    let $h: integer.unsigned.64_bit = $x plus.wrap $x;                     # wraps at u32, 'x''s size: 3705032704
    let $a: integer.signed.8_bit = 100;
    let $b: integer.signed.16_bit = 1000;
    let $w := to.integer.signed.16_bit($a plus $b) plus.wrap 1;            # a mix of sizes wraps where you say: 1101
    std::print.shortest($below, " ", $h, " ", $w, newline);
!end of MAIN
