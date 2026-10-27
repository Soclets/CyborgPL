!MAIN:
    let $x: float.binary.64_bit = 0.5;
    let $n := to.shortest.integer.signed.32_bit[$x] otherwise 0;
!end of MAIN
