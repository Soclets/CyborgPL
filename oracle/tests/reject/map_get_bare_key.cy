# a lookup only looks at its key, so a bare name says `lend` (owner, 2026-10-25)
!MAIN:
    let $m: std::map[string.utf8, integer.signed.32_bit] = std::map.empty;
    let $k: string.utf8 = "a";
    let $v := std::map.get(lend $m, key: $k) otherwise 0;
!end of MAIN
