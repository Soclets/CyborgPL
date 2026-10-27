!MAIN:
    let $xs: std::list[integer.signed.32_bit] = [1, 2, 3];
    let $z := std::list.slice(lend $xs, position: 1, position: 2);
!end of MAIN
