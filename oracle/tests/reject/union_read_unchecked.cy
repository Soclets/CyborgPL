%layout.C union Number:
    i: int.sign.32;
    f: float.bin.32;
end of %layout.C union Number

!MAIN:
    let $n := Number[i = 1];
    let $x := $n.i;
!end of MAIN
