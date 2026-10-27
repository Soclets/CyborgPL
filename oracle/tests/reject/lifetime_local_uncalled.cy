# a borrow of a local is refused at its `return`, even with no call (check claude, 2026-09-29)
function b1 [lent $a: std::list[integer.signed.32_bit]] -> lent std::list[integer.signed.32_bit]:
    let $loc: std::list[integer.signed.32_bit] = [1, 2];
    return lend $loc;
end of function b1
!MAIN:
!end of MAIN
