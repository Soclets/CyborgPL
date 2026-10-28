MAIN:
    let $xs: std::ls[i32] := [1, 2];
    for $x in lend $xs:
        change $x += 1;
    ;for
;MAIN
