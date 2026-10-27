%layout.C union Number:
    i: integer.signed.32_bit, f: float.binary.32_bit
end of layout

!MAIN:
    let $n := Number[i = 1];
    let $x := $n.i;
!end of MAIN
