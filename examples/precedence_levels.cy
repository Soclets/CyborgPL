# A kit gives its operators a precedence (owner, 2026-10-03): `precedence.low_first` counts level 1
# as the loosest (good practice), each level says how a chain reads, and a language operator in a
# level pins it where that operator is. std's `pow` binds above a negative, right pair first.
rec Vec:
    x: float.bin.64;
    y: float.bin.64;
end of rec Vec

precedence.low_first:
    level 1 [ eq, ~= ] refused
    level 2 [ plus ] left_first
    level 3 [ times, dot ] left_first
    level 4 [ scale ] left_first
end of precedence.low_first

op dot [$a: Vec, $b: Vec] -> float.bin.64:
    ret ($a.x times $b.x) plus ($a.y times $b.y);
end of op dot

op scale [$a: Vec, $k: float.bin.64] -> Vec:
    ret Vec[x = $a.x times $k, y = $a.y times $k];
end of op scale

op ~= [$a: float.bin.64, $b: float.bin.64] -> bool:
    ret ($a minus $b) less_than 0.000001 and ($b minus $a) less_than 0.000001;
end of op ~=

!MAIN:
    let $u := Vec[x = 1.0, y = 2.0];
    let $v := Vec[x = 3.0, y = 4.0];
    let $d := $u dot $v plus 1.0;          # ('u' dot 'v') plus 1.0
    let $w := $u scale 2.0 dot $v;         # ('u' scale 2.0) dot 'v'
    let $ok := $d ~= 12.0;
    std::print.shortest($d, " ", $w, " ", $ok, newln);
    std::print.shortest(2 pow 3 plus 1, " ", 2 pow 3 pow 2, " ", neg 2 pow 2, " ", 2 times 3 pow 2, newln);
!end of MAIN
