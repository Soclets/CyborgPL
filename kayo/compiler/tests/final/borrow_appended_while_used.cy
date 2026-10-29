# An append changes the list (check claude, 2026-09-29): while 'r', a borrow
# of 's' that `first` handed back, is still used, 1000 appends would move
# 's''s storage from under it — CCA's binary read freed memory. Refused.
fn first [lent $a: std::ls[i32]] -> lent std::ls[i32]:
    ret lend $a;
;fn first
MAIN:
    var $s: std::ls[i32] = [1, 2, 3];
    let $r := first(lend $s);
    for $k: i32 [from: 1, to: 1000, by: 1]:
        std::ls.append(lend.mut $s, value: $k);
    ;for
    let $n := std::ls.len(lend $r);
;MAIN
