!MAIN:
    let $p: float.%decimal.64_bit = exactly 19.99;
    let $b: float.%decimal.64_bit[maximum_places: 2] = $p times exactly 0.07;
!end of MAIN
