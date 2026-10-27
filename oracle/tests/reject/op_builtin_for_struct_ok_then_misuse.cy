rec V:
    var x: int.sign.32;
end of rec V
op plus [$a: V, $b: V] -> V:
    ret V[x = $a.x plus $b.x];
end of op plus
!MAIN:
    let $u := V[x = 1];
    let $w := $u plus 1;
!end of MAIN
