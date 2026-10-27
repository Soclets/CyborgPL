%layout.C union Number:
    i: integer.signed.32_bit;
    f: float.binary.32_bit;
end of union Number

!MAIN:
    let $n := Number[i = 1, f = 2.0];
!end of MAIN
