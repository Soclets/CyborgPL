# `runnable` has no `mean`: a path picks only that kit's operator
!MAIN:
    let $x: float.binary.64_bit = 3.0;
    let $m := $x runnable::mean $x;
!end of MAIN
