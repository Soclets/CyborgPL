# An empty value worked out at run time, handed to a length-bounded parameter (cord claude, 2026-10-14):
# with an argument it stops at the call, as the oracle does; without one it prints 1.
fn first [lent $xs: std::ls[i64][min: 1]] -> u64:
    ret std::ls.len(lend $xs);
;fn first

MAIN [$argc: i32, $argv: ptr[ptr[u8]]]:
    var $w: std::ls[i64] = [1,];
    if $argc > 1:
        change $w to std::ls.empty;
    ;if
    std::print("before", newln);
    std::print(first(lend $w), newln);
;MAIN
