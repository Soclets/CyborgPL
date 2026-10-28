# A rounded bin is the type that went in, width and all: the f16 nearest 2.7 (2.69921875 stored), which prints `2.7`.
MAIN:
    let $h: f16 := 2.675;
    std::println(std::round.near($h, places: 1));
    let $w: f32 := 2.675;
    std::println(std::round.near($w, places: 2));
    let $d: f64 := 2.675;
    std::println(std::round.near($d, places: 2));
;MAIN
