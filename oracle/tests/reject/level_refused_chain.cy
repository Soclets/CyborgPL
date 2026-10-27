operator ~= [$a: float.binary.64_bit, $b: float.binary.64_bit] -> boolean:
    return $a equal_to $b;
end of operator ~=
precedence.low_first:
    level 1 [ equal_to, ~= ] refused
end of precedence.low_first
!MAIN:
    let $x: float.binary.64_bit = 1.0;
    let $k := $x ~= $x ~= $x;
!end of MAIN
