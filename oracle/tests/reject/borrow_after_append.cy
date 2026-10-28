# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
fn first [lent $a: std::ls[i32]] -> lent std::ls[i32]:
    ret lend $a;
;fn first
MAIN:
    var $s: std::ls[i32] = [1, 2, 3];
    let $r := first(lend $s);
    std::ls.append(lend.mut $s, value: 4);
    std::print(std::ls.len(lend $r), newln);
;MAIN
