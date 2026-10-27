record Point:
    x: integer.signed.32_bit; y: integer.signed.32_bit;
end of record Point
!MAIN:
    variable $p := Point[x = 1, y = 2];
    for $v in lend.mutable $p:
    end of for
!end of MAIN
