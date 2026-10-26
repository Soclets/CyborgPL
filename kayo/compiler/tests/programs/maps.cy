%extern.C fn exit [$code: i32];
MAIN {
    let mut $phones: std::map[str.utf8, i32] = std::map["ann" = 51, "bob" = 52];
    std::map.set(lend.mut $phones, key: "cat", value: 53);
    std::map.set(lend.mut $phones, key: "ann", value: 60);
    let $a := std::map.get(lend $phones, key: "ann") otherwise 0;
    let $c := std::map.get(lend $phones, key: "cat") otherwise 0;
    let $z := std::map.get(lend $phones, key: "zed") otherwise 7;
    let $n: i32 = std::map.length(lend $phones);
    let $has: bool = std::map.has(lend $phones, key: "bob");
    let mut $h: i32 = 0;
    if $has { change $h = 1; }
    no_check my.exit($a plus $c plus $z plus $n plus $h);
}
