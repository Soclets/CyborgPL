# `pow`, and how it groups (to the right).
MAIN {
    std::print.useful((2 xx (3 xx 2)), newln);
    std::print.useful(((2 xx 3) xx 2), newln);
    let $kilo := 2 xx 10;
    std::print.useful($kilo, newln);
    std::print.useful((10 - 3 - 2), newln);
}
