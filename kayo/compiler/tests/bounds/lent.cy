# An empty value worked out at run time, handed to a length-bounded parameter (cord claude, 2026-10-14):
# with an argument it stops at the call, as the oracle does; without one it prints 1.
fn first [lent $part: str.utf8[min: 1 bytes]] -> u64:
    ret std::str.bytes(lend $part);
;fn first

MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    var $w: str.utf8 = "x";
    if $argc > 1:
        change $w to str.empty;
    ;if
    std::print("before", newln);
    std::print(first(lend $w), newln);
;MAIN
