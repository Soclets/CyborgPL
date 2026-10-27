# `for para` forces a loop parallel, `for seq` forces it in order (owner, 2026-09-30); without either,
# the prover decides, and a loop that prints stays in order by itself. The oracle runs every loop in order.
!MAIN:
    let $xs: std::list[integer.signed.64_bit] = [1, 2, 3, 4];
    variable $total: integer.signed.64_bit = 0;
    for %parallel $x in lend $xs:
        change $total = $total plus $x;
    end of for
    for sequential $x in lend $xs:
        std::print.stored($x, " ");
    end of for
    std::print.stored($total, newline);
!end of MAIN
