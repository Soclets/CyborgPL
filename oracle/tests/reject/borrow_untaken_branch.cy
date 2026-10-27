# the borrow rule holds on every path, not only the one a run takes (check claude, 2026-09-29)
!MAIN:
    let $n: int.sign.32 := 3;
    var $t: std::ls[int.sign.32] := [5,];
    if $n greater_than 5:
        let $q := lend $t;
        std::ls.append(lend.mut $t, value: 1);
        std::print.shortest(std::ls.len(lend $q), newln);
    end of if
!end of MAIN
