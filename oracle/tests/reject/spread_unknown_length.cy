function sum [...$nums: integer.signed.32_bit] -> integer.signed.32_bit:
    return 0;
end of function sum
!MAIN:
    variable $zs := [1, 2];
    std::list.append(lend.mutable $zs, value: 3);      # no bound: length no longer known here
    let $t := sum(...$zs);
!end of MAIN
