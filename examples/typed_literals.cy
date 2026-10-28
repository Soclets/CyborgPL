# A typed literal: `5: u8` (owner, 2026-10-17: "yea", and "(a)": no brackets). The `:` covers only the
# literal right before it, as it covers only the name in `$x: u8`; a variable is converted with `to.…[…]`.
MAIN:
    # a shift's count gives nothing inside it a width, so the inner shift says its own
    let $r: i64 := to.i64(1) shl (5: u8 shr 1);
    std::println($r);
    let $a := 200: u8;
    std::println($a + 55);
    let $n := -128: i8;
    std::println($n);
    std::println(3: u8 x 2);
    let $d := exactly 0.1: %d64;
    std::println($d + exactly 0.2);
    let $small := exactly 2.5: %d32;
    std::println($small);
    let $p := 5: u64[min: 1];
    std::println($p);
;MAIN
