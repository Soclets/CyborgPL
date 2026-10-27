function sum [...$n: integer.signed.32_bit] -> integer.signed.32_bit:
    return 0;
end of function sum
!MAIN:
    let $x := [1, 2]; let $t := sum(...copy $x);
!end of MAIN
