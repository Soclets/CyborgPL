!MAIN:
    let $m: map[str.utf8, int.sign.32] = map.empty;
    let $n := map.get(lend $m, value: "a") otherwise 0;
!end of MAIN
