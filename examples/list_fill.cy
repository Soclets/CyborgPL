# A list of n copies of one value (owner, 2026-09-30): `std::ls.fill(value: …, count: …)`, the count
# labelled as a position says `pos`.
MAIN:
    var $scores = std::ls.fill(value: 0, count: 5);
    change $scores[pos: 2] to 7;
    std::print.stored($scores, newln);
    let $n: i64 := 3;
    let $flags: std::ls[bool] := std::ls.fill(value: false, count: $n);
    std::print.stored($flags, " ", std::ls.len(lend $flags), newln);
    let $none: std::ls[i32] := std::ls.fill(value: 1, count: 0);
    std::print.stored(std::ls.len(lend $none), newln);
;MAIN
