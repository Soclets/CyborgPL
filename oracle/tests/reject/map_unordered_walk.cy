MAIN {
    let mut $m: std::map.unordered[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $m, key: "a", value: 1);
    for $k in std::map.keys(lend $m) { }
}
