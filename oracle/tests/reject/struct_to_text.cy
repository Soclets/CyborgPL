rec Point:
    var x: i32;
    var y: i32;
;rec Point
MAIN:
    let $p := Point[x = 3, y = 4];
    let $t := to.str.utf8($p);
;MAIN
