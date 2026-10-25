# Two maps walked by `map.pairs`, one of texts to a struct holding a text, one of texts to `i32`s: each walk's
# `pair` is its own type, and the drop made for the first was given the second (examples/night_trains: the
# count 5 was freed as a text). 1 + 5 + 6: exit 12.
struct Reach { minutes: i32, prev: str.utf8 }
MAIN {
    let $line := "a b";
    let $names := std::str.split(lend $line, at: " ");
    let $best: std::map.ordered[str.utf8, Reach] = std::map.empty;
    let $left: std::map.ordered[str.utf8, i32] = std::map.empty;
    for $n in lend $names {
        std::map.set(lend.mut $best, key: copy $n, value: Reach[minutes = 1, prev = copy $n]);
    }
    std::map.set(lend.mut $left, key: "seat", value: 5);
    std::map.set(lend.mut $left, key: "bed", value: 6);
    let mut $total: i32 = 0;
    for $e in std::map.pairs(lend $best) { if $e.key equal_to "a" { change $total = $total + $e.value.minutes; } }
    for $e in std::map.pairs(lend $left) { change $total = $total + $e.value; }
    std::exit(code: $total);
}
