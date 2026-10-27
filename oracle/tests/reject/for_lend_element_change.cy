!MAIN:
    let $xs: std::ls[int.sign.32] = [1, 2];
    for $x in lend $xs:
        change $x = $x plus 1;
    end of for
!end of MAIN
