record V:
    a: integer.signed.32_bit;
end of record V
operator % [$p: V, $q: V] -> V:
    return $p;
end of operator %
!MAIN:
!end of MAIN
