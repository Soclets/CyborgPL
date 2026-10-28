MAIN:
    let $m: std::map[str.utf8, i32] := std::map.empty; std::map.set(lend $m, key: "a", value: 1);
;MAIN
