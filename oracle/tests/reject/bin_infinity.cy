# a bin never holds an infinity (owner, 2026-09-29: a)
!MAIN:
    let $big: float.binary.64_bit = 10.0 pow 308.0;
    let $inf: float.binary.64_bit = $big times 10.0;
!end of MAIN
