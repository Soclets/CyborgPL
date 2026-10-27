!MAIN:
    variable $m: std::map[string.utf8, integer.signed.32_bit[minimum: 0, maximum: 10]][minimum: 0, maximum: 5] = std::map.empty; std::map.set(lend.mutable $m, key: "a", value: 50);
!end of MAIN
