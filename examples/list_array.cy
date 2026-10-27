# A list and an array are different types; nothing converts by itself (2026-09-19).
# `to.array(…)` freezes a list's length, `to.list(…)` lets an array grow (owner, 2026-09-23).
# The element type stays, so it is not written. `give` hands the elements over; `copy` keeps the original.
!MAIN:
    var $l: std::ls[int.sign.32] = [1, 2];
    std::ls.append(lend.mut $l, value: 3);
    let $a := to.arr(give $l);                 # [1, 2, 3], can no longer grow
    var $m := to.ls(copy $a);              # 'a' is still here
    std::ls.append(lend.mut $m, value: 4);
    var $s: std::ls[int.sign.32] = arr.slice(lend $a, from: pos 2, to: pos 3);   # a slice is a list, an array's too
    std::ls.append(lend.mut $s, value: 9);
    std::print.shortest($a, " ", $m, " ", $s, newln);
!end of MAIN
