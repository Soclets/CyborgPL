# One list may be lent to two parameters at once (both only read it). `pick` hands back 'b', which
# is in @x, so the result lives while 's' does (found by check claude, 2026-09-29).
function pick [@x] [lent $a: std::list[integer.signed.32_bit], lent @x $b: std::list[integer.signed.32_bit]] -> lent @x std::list[integer.signed.32_bit]:
    return lend $b;
end of function pick
!MAIN:
    let $s: std::list[integer.signed.32_bit] = [1, 2];
    let $r := pick(lend $s, lend $s);
    std::print.shortest(std::list.length(lend $r), newline);
!end of MAIN
