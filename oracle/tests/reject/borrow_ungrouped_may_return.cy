# The result borrows both inputs, being in one group (since 2026-10-07 a function borrowing more than
# one input says which with a group). This run returns 'a', but 'b' could have been returned, so
# giving 't' away is refused.
fn longer [@x] [lent @x $a: std::ls[i32], lent @x $b: std::ls[i32]] -> lent @x std::ls[i32]:
    if std::ls.len(lend $a) >= std::ls.len(lend $b):
        ret lend $a;
    ;if
    ret lend $b;
;fn longer
MAIN:
    let $s: std::ls[i32] := [1, 2, 3];
    let $t: std::ls[i32] := [4,];
    let $r := longer(lend $s, lend $t);
    let $g := give $t;
    std::println(std::ls.len(lend $r));
;MAIN
