%external.C procedure qsort [$base: pointer.mutable[integer.signed.32_bit], $n: integer.unsigned.64_bit, $size: integer.unsigned.64_bit, $cmp: function[pointer[integer.signed.32_bit], pointer[integer.signed.32_bit]] -> integer.signed.32_bit];
function one [$a: pointer[integer.signed.32_bit]] -> integer.signed.32_bit:
    return 0;
end of function one
!MAIN:
    variable $nums := [3, 1, 2];
    !no_check qsort(lend.mutable $nums, 3, 4, one);
!end of MAIN
