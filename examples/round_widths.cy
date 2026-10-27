# A rounded bin is the type that went in, width and all: the f16 nearest 2.7 (2.69921875 stored), which prints `2.7`.
!MAIN:
    let $h: float.binary.16_bit = 2.675;
    std::print.shortest(std::round.nearest($h, places: 1), newline);
    let $w: float.binary.32_bit = 2.675;
    std::print.shortest(std::round.nearest($w, places: 2), newline);
    let $d: float.binary.64_bit = 2.675;
    std::print.shortest(std::round.nearest($d, places: 2), newline);
!end of MAIN
