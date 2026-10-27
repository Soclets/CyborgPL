rec V:
    var x: int.sign.32;
end of rec V
op <+> [$a: V, $b: V] -> V:
    ret V[x = $a.x plus $b.x];
end of op <+>
!MAIN:
    let $u := V[x = 1];
    let $w := $u <+> $u <+> $u;
!end of MAIN
