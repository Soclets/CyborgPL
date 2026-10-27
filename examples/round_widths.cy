# A rounded bin is the type that went in, width and all: the f16 nearest 2.7 (2.69921875 stored), which prints `2.7`.
!MAIN:
    let $h: float.bin.16 := 2.675;
    std::print.shortest(std::round.near($h, places: 1), newln);
    let $w: float.bin.32 := 2.675;
    std::print.shortest(std::round.near($w, places: 2), newln);
    let $d: float.bin.64 := 2.675;
    std::print.shortest(std::round.near($d, places: 2), newln);
!end of MAIN
