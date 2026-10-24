# One list may be lent to two parameters at once (both only read it). `pick` hands back 'b', which
# is in @x, so the result lives while 's' does (found by check claude, 2026-09-29).
fn pick [@x] [lent $a: std::list[i32], lent @x $b: std::list[i32]] -> lent @x std::list[i32] {
    return lend $b;
}
MAIN {
    let $s: std::list[i32] = [1, 2];
    let $r := pick(lend $s, lend $s);
    std::print.useful(std::list.length(lend $r), newln);
}
