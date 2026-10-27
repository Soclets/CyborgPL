# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
function first [lent $a: std::list[integer.signed.32_bit]] -> lent std::list[integer.signed.32_bit]:
    return lend $a;
end of function first
!MAIN:
    variable $s: std::list[integer.signed.32_bit] = [1, 2, 3];
    let $r := first(lend $s);
    std::list.append(lend.mutable $s, value: 4);
    std::print.shortest(std::list.length(lend $r), newline);
!end of MAIN
