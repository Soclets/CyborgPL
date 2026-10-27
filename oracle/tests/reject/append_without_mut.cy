!MAIN:
    let $xs: std::ls[int.sign.32] = [1, 2, 3];
    std::ls.append(lend $xs, value: 4);
!end of MAIN
