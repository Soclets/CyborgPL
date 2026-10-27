!MAIN:
    let $xs: std::ls[int.sign.32] := [1, 2, 3];
    let $z := std::ls.slice(lend $xs, pos: 1, pos: 2);
!end of MAIN
