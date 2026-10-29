# The same as uses_board, its output taking a chunk a call (owner,
# 2026-10-09: "both").
rec Point:
    var x: i32;
    var y: i32;
;rec Point

MAIN:
    let $p := Point[x = 3, y = -4];
    let $tenth: f64 := 0.1;
    std::print("hello from a bare chip", newln);
    std::print($p, " ", $tenth, " ", 7 x 6, newln);
    std::print.stored($tenth, newln);
    std::exit(code: 0);
;MAIN
