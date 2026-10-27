!MAIN:
    variable $ys: std::list[integer.signed.32_bit][minimum: 0, maximum: 3] = [1, 2, 3];
    std::list.append(lend.mutable $ys, value: 4);
!end of MAIN
