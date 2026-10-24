# The base keeps a way to write numbers like the hardware, never the default (owner, 2026-10-06):
# `to.T.bits(…)` reads a value's bits as T's.

MAIN {
    let $a := to.i8.bits(128);        # 128's bits in an i8: -128
    let $b := to.u8.bits(-1);        # -1's bits in a u8: 255
    let $c: u8 = 200;
    let $d := to.i8.bits($c);        # 200's bits: -56
    std::print.useful($a, " ", $b, " ", $d, newln);
}
