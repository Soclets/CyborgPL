!MAIN:
    var $xs: std::ls[int.sign.32] = [1, 2, 3];
    for $x in lend.mut $xs:
        let $first := $xs[pos: 1];
        change $x = $x plus $first;
    end of for
!end of MAIN
