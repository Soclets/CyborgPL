# shift_loses_every_bit: a shift right by the whole width.
fn f [let $k: u8] -> u8:
    let $x: u8 := 1;
    ret $x shr $k;
;fn f
MAIN:
    std::print(f(9), newln);
;MAIN
