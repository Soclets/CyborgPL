# Two counters moving together (2026-10-29): `$j - $i + 1` after the loop is never 0 only by the invariant
# `$i = $j`, which no shape says. Spacer's measurement keeps it as its smallest case.
fn gap [$n: i32] -> i32:
    var $i: i32 = 0;
    var $j: i32 = 0;
    while $i < $n:
        change $i to $i + 1;
        change $j to $j + 1;
    ;while
    ret 100 div.floor ($j - $i + 1);
;fn gap
MAIN [$argc: i32, $argv: ptr[ptr[u8]]]:
    std::print(gap($argc));
;MAIN
