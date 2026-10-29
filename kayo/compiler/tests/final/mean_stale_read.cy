# Not an average: one element read once, before the loop, and added on
# every trip — the count is the input's, not the list's length. The sum
# over length template must not fit (its read has to be in the loop, at the
# current position), so the final build refuses the narrowing.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
fn stale [lent $xs: std::ls[i32], let $n: i32] -> i32:
    var $total = 0;
    let $v := lend $xs;
    var $i = 1;
    let $x := $v[pos: $i];
    while $i <= $n:
        change $total to $total + $x;
        change $i to $i + 1;
    ;while
    ret $total div.floor std::ls.len(lend $xs);
;fn stale
MAIN:
    let $k := "N";
    no_check let $n: i32 := atoi(getenv(lend $k));
    let $xs: std::ls[i32] := [2147483647,];
    no_check my.exit(stale(lend $xs, $n) mod.floor 256);
;MAIN
