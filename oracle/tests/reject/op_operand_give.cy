rec V:
    var x: int.sign.32;
end of rec V
op <*> [give $a: V, lent $b: int.sign.32] -> V:
    ret give $a;
end of op <*>
!MAIN:
!end of MAIN
