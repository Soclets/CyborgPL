function one [$a: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a;
end of function one
!MAIN:
    let $xs := [1, 2]; let $t := one(...$xs);
!end of MAIN
