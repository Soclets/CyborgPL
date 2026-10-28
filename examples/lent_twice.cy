# One list may be lent to two parameters at once (both only read it). `pick` hands back 'b', which
# is in @x, so the result lives while 's' does (found by check claude, 2026-09-29).
fn pick [@x] [lent $a: std::ls[i32], lent @x $b: std::ls[i32]] -> lent @x std::ls[i32]:
    ret lend $b;
;fn pick
MAIN:
    let $s: std::ls[i32] := [1, 2];
    let $r := pick(lend $s, lend $s);
    std::println(std::ls.len(lend $r));
;MAIN
