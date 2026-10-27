op ~= [lent $a: float.bin.64, lent $b: float.bin.64] -> bool:
    ret $a eq $b;
end of op ~=
precedence.low_first:
    level 1 [eq, ~=] refused;
end of precedence.low_first
!MAIN:
    let $x: float.bin.64 := 1.0;
    let $k := $x ~= $x ~= $x;
!end of MAIN
