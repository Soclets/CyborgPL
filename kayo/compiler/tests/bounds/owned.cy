# An empty value worked out at run time, handed to a length-bounded parameter (cord claude, 2026-10-14):
# with an argument it stops at the call, as the oracle does; without one it prints 1.
fn first [$part: str.utf8[min: 1 bytes]] -> u64 { return std::str.bytes(lend $part); }

MAIN [$argc: i32, $argv: ptr[ptr[u8]]] {
    let mut $w: str.utf8 = "x";
    if $argc > 1 { change $w = str.empty; }
    std::print.useful("before", newln);
    std::print.useful(first(give $w), newln);
}
