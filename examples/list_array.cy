# A list and an array are different types; nothing converts by itself (2026-09-19).
# `to.array(…)` freezes a list's length, `to.list(…)` lets an array grow (owner, 2026-09-23).
# The element type stays, so it is not written. `give` hands the elements over; `copy` keeps the original.
!MAIN:
    variable $l: std::list[integer.signed.32_bit] = [1, 2];
    std::list.append(lend.mutable $l, value: 3);
    let $a := to.array(give $l);                 # [1, 2, 3], can no longer grow
    variable $m := to.list(copy $a);              # 'a' is still here
    std::list.append(lend.mutable $m, value: 4);
    variable $s: std::list[integer.signed.32_bit] = array.slice(lend $a, from: position 2, to: position 3);   # a slice is a list, an array's too
    std::list.append(lend.mutable $s, value: 9);
    std::print.shortest($a, " ", $m, " ", $s, newline);
!end of MAIN
