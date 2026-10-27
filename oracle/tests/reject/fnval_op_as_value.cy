operator <+> [$a: integer.signed.32_bit, $b: integer.signed.32_bit] -> integer.signed.32_bit:
    return $a plus $b;
end of operator <+>
!MAIN:
    let $f := <+>;
!end of MAIN
