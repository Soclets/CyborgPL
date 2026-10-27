!MAIN:
    var $xs: std::ls[int.sign.32] = [1, 2, 3];
    for $x in lend.mut $xs:
        change $x = $x plus 1;
        let $n := std::ls.len(lend $xs);
        std::print.stored($x, newln);
    end of for
!end of MAIN
