# A bare literal takes the family of its context (types.md), inside a list too: each value lands in
# an element slot. `list.fill`'s value is the same. Here 1 and 2 are bins, 0.25 a d64.
!MAIN:
    let $bs: std::list[float.binary.64_bit] = [1, 2];
    let $more: std::list[float.binary.64_bit] = std::list.fill(value: 1, count: 3);
    let $ds: std::list[float.%decimal.64_bit] = std::list.fill(value: 0.25, count: 3);
    std::print.stored($bs, newline);
    std::print.stored($more, newline);
    std::print.stored($ds, newline);
!end of MAIN
