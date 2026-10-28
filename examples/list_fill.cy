# A list of n copies of one value (owner, 2026-09-30): `list.fill(value value, count count)`. A strict file
# labels the count, `list.fill(value 0, count 10)`, as a position says `pos`.
MAIN:
    var $scores = std::ls.fill(value: 0, count: 5);
    change $scores[pos: 2] to 7;
    std::println.stored($scores);
    let $n: i64 := 3;
    let $flags: std::ls[bool] := std::ls.fill(value: false, count: $n);
    std::println.stored($flags, " ", std::ls.len(lend $flags));
    let $none: std::ls[i32] := std::ls.fill(value: 1, count: 0);
    std::println.stored(std::ls.len(lend $none));
;MAIN
