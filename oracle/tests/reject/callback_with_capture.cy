%external.C procedure qsort [$base: pointer.mutable[integer.signed.32_bit], $n: integer.unsigned.64_bit, $size: integer.unsigned.64_bit, $cmp: function[pointer[integer.signed.32_bit], pointer[integer.signed.32_bit]] -> integer.signed.32_bit];
!MAIN:
    variable $nums := [3, 1, 2];
    let $flip := true;
    !no_check qsort(lend.mutable $nums, 3, 4, function [$a: pointer[integer.signed.32_bit], $b: pointer[integer.signed.32_bit]] -> integer.signed.32_bit with [lend $flip]:
        return 0;
    end of function);
!end of MAIN
