%extern.C proc exit [$code: i32];
MAIN:
    var $m: std::map[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $m, key: "seven", value: 7);
    std::map.set(lend.mut $m, key: "three", value: 3);
    std::map.set(lend.mut $m, key: "five", value: 5);
    std::map.set(lend.mut $m, key: "three", value: 9);
    var $acc: i32 = 0;
    for $v in std::map.values(lend $m):
        change $acc to ($acc x 3) + $v;
    ;for
    var $keylen: i32 = 0;
    for $k in std::map.keys(lend $m):
        change $keylen to $keylen + std::str.bytes(lend $k);
    ;for
    no_check my.exit($acc + $keylen);
;MAIN
