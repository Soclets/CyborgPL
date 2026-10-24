# A file's `precedence` block (owner, 2026-10-03): `low_first` counts level
# 1 as the loosest; a language operator pins its level, and a level with
# none sits between its neighbours. Here `dot` binds as `times` does, `gap`
# sits between `plus` and `times`, `~=` as `==` and refused in a chain;
# std's `pow` sits above a negative, right pair first.
#   'u' dot 'v' plus 1 = (3 x 4 + 1) = 13
#   20 gap 3 times 2 = 20 - 6 = 14;  20 gap 3 plus 2 = 17 + 2 = 19
#   -2 pow 2 = -4;  2 pow 3 pow 2 = 512 (mod 256 is 0);  2 times 3 pow 2 = 18
# 13 + 14 + 19 - 4 + 0 + 18 + 1 (the ~= holds) = 61.
extern.C fn exit [$code: i32];

precedence.low_first {
    level 1 { equal_to, ~= } refused
    level 2 { plus } left_first
    level 3 { gap } left_first
    level 4 { times, dot } left_first
}

op dot [$a: i32, $b: i32] -> i32 { return $a times $b; }
op gap [$a: i32, $b: i32] -> i32 { return $a minus $b; }
op ~= [$a: i32, $b: i32] -> bool { return ($a minus $b) < 2 and ($b minus $a) < 2; }

MAIN {
    let $u: i32 = 3;
    let $v: i32 = 4;
    let $p := $u dot $v plus 1;
    let $q := 20 gap 3 times 2;
    let $r := 20 gap 3 plus 2;
    let $s: i32 = -2 pow 2;
    let $t: i32 = (2 pow 3 pow 2) mod.floor 256;
    let $w: i32 = 2 times 3 pow 2;
    let mut $k: i32 = 0;
    if $p ~= 13 { change $k = 1; }
    no_check my.exit($p plus $q plus $r plus $s plus $t plus $w plus $k);
}
