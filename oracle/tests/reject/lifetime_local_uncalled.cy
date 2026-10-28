# a borrow of a local is refused at its `return`, even with no call (check claude, 2026-09-29)
fn b1 [lent $a: std::ls[i32]] -> lent std::ls[i32]:
    let $loc: std::ls[i32] := [1, 2];
    ret lend $loc;
;fn b1
MAIN:
;MAIN
