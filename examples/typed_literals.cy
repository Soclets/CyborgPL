# A typed literal: `5: u8` (owner, 2026-10-17: "yea", and "(a)": no brackets). The `:` covers only the
# literal right before it, as it covers only the name in `$x: u8`; a variable is converted with `to.…[…]`.
!MAIN:
    # a shift's count gives nothing inside it a width, so the inner shift says its own
    let $r: int.sign.64 := to.int.sign.64(1) shl (5: int.unsign.8 shr 1);
    std::print.shortest($r, newln);
    let $a := 200: int.unsign.8;
    std::print.shortest($a plus 55, newln);
    let $n := neg 128: int.sign.8;
    std::print.shortest($n, newln);
    std::print.shortest(3: int.unsign.8 times 2, newln);
    let $d := exactly 0.1: float.%deci.64;
    std::print.shortest($d plus exactly 0.2, newln);
    let $small := exactly 2.5: float.%deci.32;
    std::print.shortest($small, newln);
    let $p := 5: int.unsign.64[min: 1];
    std::print.shortest($p, newln);
!end of MAIN
