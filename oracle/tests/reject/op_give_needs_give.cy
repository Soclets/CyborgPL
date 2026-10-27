rec V:
    x: int.sign.32;
end of rec V
op <*> [give $a: V, $b: int.sign.32] -> V:
    ret give $a;
end of op <*>
!MAIN:
    let $u := V[x = 1];
    let $w := $u <*> 2;
!end of MAIN
