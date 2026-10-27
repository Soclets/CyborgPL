!MAIN:
    let $m: std::map[string.utf8, integer.signed.32_bit] = std::map.empty; let $x := std::map.get(lend $m, key: "a");
!end of MAIN
