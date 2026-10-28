proc grow [@y] [lent.mut @y $xs: std::ls[i32]] -> lent.mut @y std::ls[i32]:
    std::ls.append(lend.mut $xs, value: 9); ret lend.mut $xs;
;proc grow
proc two [lent.mut $a: std::ls[i32], lent $b: std::ls[i32]]:
    std::print(std::ls.len(lend $a) + std::ls.len(lend $b), newln);
;proc two
MAIN:
    var $u: std::ls[i32] = [1, 2];
    two(grow(lend.mut $u), lend $u);
;MAIN
