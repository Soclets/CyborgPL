%extern.C proc exit [$code: i32];

rec Point:
    var x: i32; var y: i32;
;rec Point

fn area [lent $p: Point] -> i32:
    ret $p.x x $p.y;
;fn area

MAIN:
    let $p := Point[x = 6, y = 7];
    no_check my.exit(area(lend $p));                     # 42
;MAIN
