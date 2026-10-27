record V:
    x: integer.signed.32_bit;
end of record V
operator plus [$a: V, $b: V] -> V:
    return V[x = $a.x plus $b.x];
end of operator plus
!MAIN:
    let $u := V[x = 1];
    let $w := $u plus 1;
!end of MAIN
