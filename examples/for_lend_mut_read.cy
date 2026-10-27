# Inside `for $x in lend.mut $xs`, the list may be read once 'x' is used for the last time in
# that trip, and the read sees the new values (owner, 2026-09-30: a, kayo's liveness rule).
!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [1, 2, 3];
    variable $seen: std::list[integer.signed.32_bit] = [];
    for $x in lend.mutable $xs:
        change $x = $x times 10;
        std::list.append(lend.mutable $seen, value: $xs[position: 1]);
    end of for
    std::print.stored($xs, newline);
    std::print.stored($seen, newline);
!end of MAIN
