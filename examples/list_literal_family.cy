# A bare literal takes the family of its context (types.md), inside a list too: each value lands in
# an element slot. `list.fill`'s value is the same. Here 1 and 2 are bins, 0.25 a d64.
!MAIN:
    let $bs: std::ls[float.bin.64] := [1, 2];
    let $more: std::ls[float.bin.64] := std::ls.fill(value: 1, count: 3);
    let $ds: std::ls[float.%deci.64] := std::ls.fill(value: 0.25, count: 3);
    std::print.stored($bs, newln);
    std::print.stored($more, newln);
    std::print.stored($ds, newln);
!end of MAIN
