!MAIN:
    let $p: float.%decimal.64_bit = exactly 19.99;
    let $tax := exactly 7 percent;
    let $x := $p plus $tax;
!end of MAIN
