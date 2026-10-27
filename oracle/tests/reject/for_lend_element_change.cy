!MAIN:
    let $xs: std::list[integer.signed.32_bit] = [1, 2];
    for $x in lend $xs:
        change $x = $x plus 1;
    end of for
!end of MAIN
