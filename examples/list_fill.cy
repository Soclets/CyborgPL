# A list of n copies of one value (owner, 2026-09-30): `list.fill(value value, count count)`. A strict file
# labels the count, `list.fill(value 0, count 10)`, as a position says `pos`.
!MAIN:
    var $scores = std::ls.fill(value: 0, count: 5);
    change $scores[pos: 2] to 7;
    std::print.stored($scores, newln);
    let $n: int.sign.64 := 3;
    let $flags: std::ls[bool] := std::ls.fill(value: false, count: $n);
    std::print.stored($flags, " ", std::ls.len(lend $flags), newln);
    let $none: std::ls[int.sign.32] := std::ls.fill(value: 1, count: 0);
    std::print.stored(std::ls.len(lend $none), newln);
!end of MAIN
