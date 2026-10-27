record V:
    x: integer.signed.32_bit;
end of record V
operator <*> [give $a: V, $b: integer.signed.32_bit] -> V:
    return give $a;
end of operator <*>
!MAIN:
    let $u := V[x = 1];
    let $w := $u <*> 2;
!end of MAIN
