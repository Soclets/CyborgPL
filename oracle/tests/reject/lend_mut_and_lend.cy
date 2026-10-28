proc two [lent.mut $a: std::ls[i32], lent $b: std::ls[i32]]:
    std::print(std::ls.len(lend $a) + std::ls.len(lend $b), newln);
;proc two
MAIN:
    var $u: std::ls[i32] = [1, 2];
    two(lend.mut $u, lend $u);
;MAIN
