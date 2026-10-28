MAIN:
    var $x: i32 = 1;
    var shadow $x: i32 = 2;
    change $x to 3;
    std::print($x, newln);
;MAIN
