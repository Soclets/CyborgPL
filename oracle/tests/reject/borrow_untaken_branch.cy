# the borrow rule holds on every path, not only the one a run takes (check claude, 2026-09-29)
!MAIN:
    let $n: integer.signed.32_bit = 3;
    variable $t: std::list[integer.signed.32_bit] = [5,];
    if $n greater_than 5:
        let $q := lend $t;
        std::list.append(lend.mutable $t, value: 1);
        std::print.shortest(std::list.length(lend $q), newline);
    end of if
!end of MAIN
