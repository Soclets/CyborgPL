MAIN:
    var $xs: std::ls[i32] = [1, 2, 3];
    for $x in lend.mut $xs:
        change $x += 1;
        let $n := std::ls.len(lend $xs);
        std::print.stored($x, newln);
    ;for
;MAIN
