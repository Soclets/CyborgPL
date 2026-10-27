%layout.C union Number:
    i: int.sign.32;
    f: float.bin.32;
end of union Number

!MAIN:
    let $n := Number[i = 1, f = 2.0];
!end of MAIN
