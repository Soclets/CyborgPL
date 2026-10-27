!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [1, 2, 3];
    for $x in lend.mutable $xs:
        change $x = $x plus 1;
        let $n := std::list.length(lend $xs);
        std::print.stored($x, newline);
    end of for
!end of MAIN
