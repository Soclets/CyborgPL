!MAIN:
    variable $m: std::map.unordered[string.utf8, integer.signed.32_bit] = std::map.empty;
    std::map.set(lend.mutable $m, key: "a", value: 1);
    for $k in std::map.keys(lend $m):
    end of for
!end of MAIN
