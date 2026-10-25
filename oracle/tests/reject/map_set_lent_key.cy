# `map.set` keeps its key, so a lent one is refused (2026-09-22; lookups take one lent, owner 2026-10-25)
MAIN {
    let mut $m: std::map[str.utf8, i32] = std::map.empty;
    let $k: str.utf8 = "a";
    std::map.set(lend.mut $m, key: lend $k, value: 1);
}
