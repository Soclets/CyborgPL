# a bin never holds an infinity (owner, 2026-09-29: a)
!MAIN:
    let $big: float.bin.64 := 10.0 pow 308.0;
    let $inf: float.bin.64 := $big times 10.0;
!end of MAIN
