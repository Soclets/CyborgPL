MAIN:
    let $m: std::map[str.utf8, i32] := std::map.empty;
    let $t := to.str.utf8(lend $m);
;MAIN
