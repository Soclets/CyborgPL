MAIN:
    var $m: std::map[str.utf8, i32[min: 0, max: 10]][min: 0, max: 5] = std::map.empty; std::map.set(lend.mut $m, key: "a", value: 50);
;MAIN
