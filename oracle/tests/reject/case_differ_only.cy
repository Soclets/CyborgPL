case.any [function];
function area [] -> integer.signed.32_bit:
    return 1;
end of function area
record Area:
    n: integer.signed.32_bit;
end of record Area
!MAIN:
    let $a := area();
!end of MAIN
