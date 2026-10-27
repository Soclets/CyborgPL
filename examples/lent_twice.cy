# One list may be lent to two parameters at once (both only read it). `pick` hands back 'b', which
# is in @x, so the result lives while 's' does (found by check claude, 2026-09-29).
fn pick [@x] [lent $a: std::ls[int.sign.32], lent @x $b: std::ls[int.sign.32]] -> lent @x std::ls[int.sign.32]:
    ret lend $b;
end of fn pick
!MAIN:
    let $s: std::ls[int.sign.32] = [1, 2];
    let $r := pick(lend $s, lend $s);
    std::print.shortest(std::ls.len(lend $r), newln);
!end of MAIN
