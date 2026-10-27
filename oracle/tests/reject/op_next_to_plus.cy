operator <+> [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a plus $b;
end of operator <+>
!MAIN:
    let $w := 1 plus 2 <+> 3;
!end of MAIN
