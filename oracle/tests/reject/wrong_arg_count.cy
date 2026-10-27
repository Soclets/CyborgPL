function one [$a: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a;
end of function one
!MAIN:
    let $x := one(1, 2);
!end of MAIN
