# `map.set` keeps its key, so a lent one is refused (2026-09-22; lookups take one lent, owner 2026-10-25)
!MAIN:
    variable $m: std::map[string.utf8, integer.signed.32_bit] = std::map.empty;
    let $k: string.utf8 = "a";
    std::map.set(lend.mutable $m, key: lend $k, value: 1);
!end of MAIN
