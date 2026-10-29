# unknown_field: a field a letter off.
rec P:
    var width: i32;
;rec P
MAIN:
    let $p := P[width = 2];
    std::print($p.widht, newln);
;MAIN
