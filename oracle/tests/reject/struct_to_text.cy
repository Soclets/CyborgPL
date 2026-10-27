record Point:
    x: integer.signed.32_bit;
    y: integer.signed.32_bit;
end of record Point
!MAIN:
    let $p := Point[x = 3, y = 4];
    let $t := to.string.utf8($p);
!end of MAIN
