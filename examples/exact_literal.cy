# `exactly 0.1` means "exactly this" (owner, 2026-10-02; marked `*0.1` until 2026-10-16): it is a deci; a plain one
# is a bin, meaning f64's answer. Under `:=` the oracle makes it a d64; the prover picks the width.
!MAIN:
    let $price := exactly 19.99;
    let $tenth := exactly 0.1;
    let $total := $tenth plus exactly 0.2;              # exactly 0.3
    let $half: float.%deci.32 := exactly 0.5;                  # a typed deci takes it at its width
    let $speed := 0.1;                         # a bin
    std::print.stored($price, " ", $total, " ", $half, " ", $speed, newln);
!end of MAIN
