record V:
    x: integer.signed.32_bit;
end of record V
operator <+> [$a: V, $b: V] -> V:
    return V[x = $a.x plus $b.x];
end of operator <+>
!MAIN:
    let $u := V[x = 1];
    let $w := $u <+> $u <+> $u;
!end of MAIN
