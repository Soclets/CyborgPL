# The same as uses_board, its output taking a chunk a call (owner,
# 2026-10-09: "both").
struct Point {
    x: int32,
    y: int32
}

MAIN {
    let 'p' := Point[x = 3, y = -4];
    let 'tenth': bin64 = 0.1;
    std::print.useful["hello from a bare chip", newln];
    std::print.useful['p', " ", 'tenth', " ", 7 times 6, newln];
    std::print.stored['tenth', newln];
    std::exit[0];
}
