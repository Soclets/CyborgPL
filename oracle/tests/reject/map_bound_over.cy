!MAIN:
    var $small: std::map[str.utf8[min: 1, max: 3 bytes], int.sign.32[min: 0, max: 10]][min: 0, max: 2] = std::map.empty;
    std::map.set(lend.mut $small, key: "ab", value: 5);
    std::map.set(lend.mut $small, key: "cd", value: 7);
    let $n := std::map.count(lend $small);
    let $tiny: int.sign.8 := $n;                          # proven by the bound: at most 2
    std::print.shortest($tiny, newln);
    std::map.set(lend.mut $small, key: "ef", value: 1);              # a third: does not fit [0, 2]
!end of MAIN
