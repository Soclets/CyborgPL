MAIN:
    let $xs: std::ls[i32] := [1, 2, 3];
    let $z := std::ls.slice(lend $xs, pos: 1, pos: 2);
;MAIN
