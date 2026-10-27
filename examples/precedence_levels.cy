# A kit gives its operators a precedence (owner, 2026-10-03): `precedence.low_first` counts level 1
# as the loosest (good practice), each level says how a chain reads, and a language operator in a
# level pins it where that operator is. std's `pow` binds above a negative, right pair first.
record Vec:
    x: float.binary.64_bit; y: float.binary.64_bit;
end of record Vec

precedence.low_first:
    level 1 [ equal_to, ~= ] refused
    level 2 [ plus ] left_first
    level 3 [ times, dot ] left_first
    level 4 [ scale ] left_first
end of precedence.low_first

operator dot [$a: Vec, $b: Vec] -> float.binary.64_bit:
    return ($a.x times $b.x) plus ($a.y times $b.y);
end of operator dot

operator scale [$a: Vec, $k: float.binary.64_bit] -> Vec:
    return Vec[x = $a.x times $k, y = $a.y times $k];
end of operator scale

operator ~= [$a: float.binary.64_bit, $b: float.binary.64_bit] -> boolean:
    return ($a minus $b) less_than 0.000001 and ($b minus $a) less_than 0.000001;
end of operator ~=

!MAIN:
    let $u := Vec[x = 1.0, y = 2.0];
    let $v := Vec[x = 3.0, y = 4.0];
    let $d := $u dot $v plus 1.0;          # ('u' dot 'v') plus 1.0
    let $w := $u scale 2.0 dot $v;         # ('u' scale 2.0) dot 'v'
    let $ok := $d ~= 12.0;
    std::print.shortest($d, " ", $w, " ", $ok, newline);
    std::print.shortest(2 power 3 plus 1, " ", 2 power 3 power 2, " ", negative 2 power 2, " ", 2 times 3 power 2, newline);
!end of MAIN
