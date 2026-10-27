!MAIN:
    let $m: std::map[str.utf8, int.sign.32] := std::map.empty; let $x := std::map.get(lend $m, key: "a");
!end of MAIN
