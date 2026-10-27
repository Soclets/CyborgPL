!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [1, 2, 3];
    for $x in lend.mutable $xs:
        let $first := $xs[position: 1];
        change $x = $x plus $first;
    end of for
!end of MAIN
