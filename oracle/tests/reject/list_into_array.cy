# nothing converts by itself: a list into an array is said (owner, 2026-09-23)
!MAIN:
    let $l: std::list[integer.signed.32_bit] = [1, 2];
    let $a: array[integer.signed.32_bit] = give $l;
!end of MAIN
