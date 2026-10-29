%extern.C proc exit [let $code: i32];
MAIN:
    var $phones: std::map[str.utf8, i32] = std::map["ann" = 51, "bob" = 52];
    std::map.set(lend.mut $phones, key: "cat", value: 53);
    std::map.set(lend.mut $phones, key: "ann", value: 60);
    let $a := std::map.get(lend $phones, key: "ann") ?? 0;
    let $c := std::map.get(lend $phones, key: "cat") ?? 0;
    let $z := std::map.get(lend $phones, key: "zed") ?? 7;
    let $n: i32 := std::map.count(lend $phones);
    let $has: bool := std::map.has(lend $phones, key: "bob");
    var $h: i32 = 0;
    if $has:
        change $h to 1;
    ;if
    no_check my.exit($a + $c + $z + $n + $h);
;MAIN
