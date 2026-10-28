MAIN:
    var $xs: std::ls[i32] = [1, 2, 3];
    for $x in lend.mut $xs:
        let $first := $xs[pos: 1];
        change $x to $x + $first;
    ;for
;MAIN
