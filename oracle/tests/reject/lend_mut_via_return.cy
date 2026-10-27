proc grow [@y] [lent.mut @y $xs: std::ls[int.sign.32]] -> lent.mut @y std::ls[int.sign.32]:
    std::ls.append(lend.mut $xs, value: 9); ret lend.mut $xs;
end of proc grow
proc two [lent.mut $a: std::ls[int.sign.32], lent $b: std::ls[int.sign.32]]:
    std::print.shortest(std::ls.len(lend $a) plus std::ls.len(lend $b), newln);
end of proc two
!MAIN:
    var $u: std::ls[int.sign.32] := [1, 2];
    two(grow(lend.mut $u), lend $u);
!end of MAIN
