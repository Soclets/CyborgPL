!MAIN:
    let $m: std::map[string.utf8, integer.signed.32_bit] = std::map.empty;
    let $t := to.string.utf8($m);
!end of MAIN
