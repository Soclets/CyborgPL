# The same as uses_board, its output taking a chunk a call (owner,
# 2026-10-09: "both").
struct Point {
    x: i32,
    y: i32
}

MAIN {
    let $p := Point[x = 3, y = -4];
    let $tenth: f64 = 0.1;
    std::print.useful("hello from a bare chip", newln);
    std::print.useful($p, " ", $tenth, " ", 7 times 6, newln);
    std::print.stored($tenth, newln);
    std::exit(code: 0);
}
