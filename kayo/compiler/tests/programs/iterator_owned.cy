# An iterator handing out owned lists (owner, 2026-10-09): each item is
# freed after its trip, and `perm` keeps the last one past the loop.
%extern.C proc exit [$code: i32];

rec Rows:
    var left: i32;
;rec Rows

proc Rows.next [lent.mut $r: Rows] -> optional[std::ls[i32]]:
    if $r.left == 0:
        ret optional.none;
    ;if
    change $r.left to $r.left - 1;
    var $row: std::ls[i32] = [1, 2];
    std::ls.append(lend.mut $row, value: $r.left);
    ret optional.value(give $row);
;proc Rows.next

MAIN:
    var $total: i32 = 0;
    var $r = Rows[left = 4];
    for $row in lend.mut $r:
        change $total to $total + to.i32(std::ls.len(lend $row));
    ;for
    let $s := Rows[left = 3];
    for perm $last in give $s:
        change $total to $total + 1;
    ;for
    no_check my.exit($total + to.i32(std::ls.len(lend $last)));
;MAIN
