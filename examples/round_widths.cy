# A rounded bin is the type that went in, width and all: the bin16 nearest 2.7 is 2.69921875.
MAIN {
    let 'h': bin16 = 2.675;
    std::print.useful[round.near['h', decimals 1], newln];
    let 'w': bin32 = 2.675;
    std::print.useful[round.near['w', decimals 2], newln];
    let 'd': bin64 = 2.675;
    std::print.useful[round.near['d', decimals 2], newln];
}
