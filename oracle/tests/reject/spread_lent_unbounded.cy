function sum [...$nums: integer.signed.32_bit] -> integer.signed.32_bit:
    return 0;
end of function sum
function f [lent $xs: std::list[integer.signed.32_bit]] -> integer.signed.32_bit:
    return sum(...$xs);
end of function f
!MAIN:
    let $a := [1, 2]; let $t := f(lend $a);
!end of MAIN
