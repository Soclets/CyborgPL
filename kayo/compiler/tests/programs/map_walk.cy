%extern.C fn exit [$code: i32];
MAIN {
    let mut $m: std::map[str.utf8, i32] = std::map.empty;
    std::map.set(lend.mut $m, key: "seven", value: 7);
    std::map.set(lend.mut $m, key: "three", value: 3);
    std::map.set(lend.mut $m, key: "five", value: 5);
    std::map.set(lend.mut $m, key: "three", value: 9);
    let mut $acc: i32 = 0;
    for $v in std::map.values(lend $m) {
        change $acc = ($acc times 3) plus $v;
    }
    let mut $keylen: i32 = 0;
    for $k in std::map.keys(lend $m) {
        change $keylen = $keylen plus std::str.bytes(lend $k);
    }
    no_check my.exit($acc plus $keylen);
}
