# Text → number fails `not_a_number` when it is not a number, and `out_of_range`
# when it is one too big for the type, however many digits (owner, 2026-09-24).
# Text comes from outside, so either is bad input to handle, never a crash.
MAIN {
    let $a := to.i32("42") otherwise 0;
    let $b := to.i32("99999999999") otherwise -1;                               # too big for i32
    let $c := to.i64("99999999999999999999999999999999999999999999") otherwise -1;  # too big for anything
    let $d := to.u8("-1") otherwise 0;                                         # below what u8 holds
    let $e := to.i32("12x") otherwise -2;                                      # not a number at all
    std::print.useful($a, " ", $b, " ", $c, " ", $d, " ", $e, newln);
    # text → bin: digits, an optional fraction, an optional exponent (owner, 2026-09-24: b)
    let $f := to.f64("2.5e3") otherwise -1.0;
    let $g := to.f64("nan") otherwise -1.0;      # not a number: a data file's "missing" lands here
    let $h := to.f64("1e400") otherwise -2.0;    # a number too big for f64
    std::print.useful($f, " ", $g, " ", $h, newln);
}
