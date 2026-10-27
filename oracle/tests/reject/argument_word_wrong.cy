!MAIN:
    let $m: map[string.utf8, integer.signed.32_bit] = map.empty;
    let $n := map.get(lend $m, value: "a") otherwise 0;
!end of MAIN
