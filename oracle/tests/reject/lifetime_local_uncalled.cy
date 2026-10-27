# a borrow of a local is refused at its `return`, even with no call (check claude, 2026-09-29)
fn b1 [lent $a: std::ls[int.sign.32]] -> lent std::ls[int.sign.32]:
    let $loc: std::ls[int.sign.32] = [1, 2];
    ret lend $loc;
end of fn b1
!MAIN:
!end of MAIN
