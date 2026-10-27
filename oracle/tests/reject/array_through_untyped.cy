# an array taken over by `:=` is still an array, so it cannot land in a list's place (2026-09-23)
!MAIN:
    let $a: array[integer.signed.32_bit] = [5, 6];
    let $b := give $a;
    let $c: std::list[integer.signed.32_bit] = give $b;
!end of MAIN
