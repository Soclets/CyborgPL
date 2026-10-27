!MAIN:
    let $x: float.bin.64 = 0.5;
    let $n := to.shortest.int.sign.32[$x] otherwise 0;
!end of MAIN
