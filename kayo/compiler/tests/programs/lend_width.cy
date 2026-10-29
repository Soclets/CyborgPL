# A `:=` list of bare numbers takes its width from its use (types.md:
# Precog picks the type): lent to a `list[i32]`, it is one. Left at the
# exact width, the callee read the wrong layout and summed garbage.
%extern.C proc exit [$code: i32];
fn total [lent $xs: std::ls[i32]] -> i32:
    var $n: i32 = 0;
    for $x in lend $xs:
        change $n to $n + $x;
    ;for
    ret $n;
;fn total
MAIN:
    let $scores := [72, 40, 95, 55];
    let $first := $scores[pos: 1] + 1;
    no_check my.exit((total(lend $scores) + to.i32($first)) mod.floor 256);
;MAIN
