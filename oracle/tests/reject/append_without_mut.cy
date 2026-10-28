MAIN:
    let $xs: std::ls[i32] := [1, 2, 3];
    std::ls.append(lend $xs, value: 4);
;MAIN
