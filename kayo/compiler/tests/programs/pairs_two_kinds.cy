# Two maps walked by `map.pairs`, one of texts to a struct holding a text, one of texts to `i32`s: each walk's
# `pair` is its own type, and the drop made for the first was given the second (examples/night_trains: the
# count 5 was freed as a text). 1 + 5 + 6: exit 12. The walk is a function's, so all it holds is given back.
rec Reach:
    var minutes: i32; var prev: str.utf8;
;rec Reach
fn walk [] -> i32:
    let $line := "a b";
    let $names := std::str.split(lend $line, at: " ");
    var $best: std::map.ordered[str.utf8, Reach] = std::map.empty;
    var $left: std::map.ordered[str.utf8, i32] = std::map.empty;
    for $n in lend $names:
        std::map.set(lend.mut $best, key: copy $n, value: Reach[minutes = 1, prev = copy $n]);
    ;for
    std::map.set(lend.mut $left, key: "seat", value: 5);
    std::map.set(lend.mut $left, key: "bed", value: 6);
    var $total: i32 = 0;
    for $e in std::map.pairs(lend $best):
        if $e.key == "a":
            change $total to $total + $e.value.minutes;
        ;if
    ;for
    for $e in std::map.pairs(lend $left):
        change $total to $total + $e.value;
    ;for
    ret $total;
;fn walk
MAIN:
    std::exit(code: walk());
;MAIN
