# A list of n copies of one value (owner, 2026-09-30): `list.fill(value value, count count)`. A strict file
# labels the count, `list.fill(value 0, count 10)`, as a position says `pos`.
!MAIN:
    variable $scores := std::list.fill(value: 0, count: 5);
    change $scores[position: 2] = 7;
    std::print.stored($scores, newline);
    let $n: integer.signed.64_bit = 3;
    let $flags: std::list[boolean] = std::list.fill(value: false, count: $n);
    std::print.stored($flags, " ", std::list.length(lend $flags), newline);
    let $none: std::list[integer.signed.32_bit] = std::list.fill(value: 1, count: 0);
    std::print.stored(std::list.length(lend $none), newline);
!end of MAIN
