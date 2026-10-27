# check claude's borrow findings (2026-09-29): a borrow never aliases a change and never outlives its owner (core-features.md)
fn first [lent $a: std::ls[int.sign.32]] -> lent std::ls[int.sign.32]:
    ret lend $a;
end of fn first
!MAIN:
    var $s: std::ls[int.sign.32] = [1, 2, 3];
    let $r := first(lend $s);
    std::ls.append(lend.mut $s, value: 4);
    std::print.shortest(std::ls.len(lend $r), newln);
!end of MAIN
