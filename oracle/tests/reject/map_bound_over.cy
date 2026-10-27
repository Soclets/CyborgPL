!MAIN:
    variable $small: std::map[string.utf8[minimum: 1, maximum: 3 bytes], integer.signed.32_bit[minimum: 0, maximum: 10]][minimum: 0, maximum: 2] = std::map.empty;
    std::map.set(lend.mutable $small, key: "ab", value: 5);
    std::map.set(lend.mutable $small, key: "cd", value: 7);
    let $n := std::map.count(lend $small);
    let $tiny: integer.signed.8_bit = $n;                          # proven by the bound: at most 2
    std::print.shortest($tiny, newline);
    std::map.set(lend.mutable $small, key: "ef", value: 1);              # a third: does not fit [0, 2]
!end of MAIN
