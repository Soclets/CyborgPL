# A rounded bin is the type that went in, width and all: the f16 nearest 2.7 is 2.69921875.
MAIN {
    let 'h': f16 = 2.675;
    std::print.useful[round.near['h', places: 1], newln];
    let 'w': f32 = 2.675;
    std::print.useful[round.near['w', places: 2], newln];
    let 'd': f64 = 2.675;
    std::print.useful[round.near['d', places: 2], newln];
}
