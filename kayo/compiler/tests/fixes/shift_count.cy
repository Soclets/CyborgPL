# shift_loses_every_bit: a shift right by the whole width.
fn f [$k: u8] -> u8 {
    let $x: u8 = 1;
    return $x shr $k;
}
MAIN {
    std::print.useful(f(9), newln);
}
