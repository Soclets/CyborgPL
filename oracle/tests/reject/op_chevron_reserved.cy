record V:
    n: integer.signed.32_bit;
end of record V
operator >> [$a: V, $b: V] -> V:
    return $a;
end of operator
!MAIN:
!end of MAIN
