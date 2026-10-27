# Text → number fails `not_a_number` when it is not a number, and `out_of_range`
# when it is one too big for the type, however many digits (owner, 2026-09-24).
# Text comes from outside, so either is bad input to handle, never a crash.
!MAIN:
    let $a := to.integer.signed.32_bit("42") otherwise 0;
    let $b := to.integer.signed.32_bit("99999999999") otherwise negative 1;                               # too big for i32
    let $c := to.integer.signed.64_bit("99999999999999999999999999999999999999999999") otherwise negative 1;  # too big for anything
    let $d := to.integer.unsigned.8_bit("-1") otherwise 0;                                         # below what u8 holds
    let $e := to.integer.signed.32_bit("12x") otherwise negative 2;                                      # not a number at all
    std::print.shortest($a, " ", $b, " ", $c, " ", $d, " ", $e, newline);
    # text → bin: digits, an optional fraction, an optional exponent (owner, 2026-09-24: b)
    let $f := to.float.binary.64_bit("2.5e3") otherwise negative 1.0;
    let $g := to.float.binary.64_bit("nan") otherwise negative 1.0;      # not a number: a data file's "missing" lands here
    let $h := to.float.binary.64_bit("1e400") otherwise negative 2.0;    # a number too big for f64
    std::print.shortest($f, " ", $g, " ", $h, newline);
!end of MAIN
