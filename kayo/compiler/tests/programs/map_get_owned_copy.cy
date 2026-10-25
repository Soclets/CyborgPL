# `map.get` hands back a value of its own (design claude's night_trains, 2026-10-25): a struct holding a text,
# got and dropped three times over, left the map's text freed and the map freed it again (exit 133). Texts made
# when the program runs, by `str.split`, as a static one is never freed. 3 rounds of 3 stations, 1 each: exit 9.
struct Reach { minutes: i32, prev: str.utf8 }
MAIN {
    let $line := "a b c";
    let $names := std::str.split(lend $line, at: " ");
    let $best: std::map[str.utf8, Reach] = std::map.empty;
    for $n in lend $names {
        std::map.set(lend.mut $best, key: copy $n, value: Reach[minutes = 1, prev = copy $n]);
    }
    let mut $total: i32 = 0;
    for $round [from: 1, to: 3, by: 1] {
        for $n in lend $names {
            let $r := std::map.get(lend $best, key: lend $n) otherwise { missing { continue; } };
            change $total = $total + $r.minutes;
        }
    }
    std::exit(code: $total);
}
