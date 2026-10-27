# two meanings of one operator for the same types (2026-09-23)
record A:
    x: integer.signed.32_bit;
end of record A
operator <+> [$a: A, $b: A] -> integer.signed.32_bit:
    return 1;
end of operator <+>
operator <+> [$a: A, $b: A] -> integer.signed.32_bit:
    return 2;
end of operator <+>
!MAIN:
!end of MAIN
