operator dot [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a times $b;
end of operator dot
!MAIN:
    let $x := dot(1, 2);
!end of MAIN
