function big [$a: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a times $a;
end of function big
!MAIN:
    let $r := big(100000);
!end of MAIN
