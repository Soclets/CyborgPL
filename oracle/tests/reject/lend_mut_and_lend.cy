proc two [lent.mut $a: std::ls[int.sign.32], lent $b: std::ls[int.sign.32]]:
    std::print.shortest(std::ls.len(lend $a) plus std::ls.len(lend $b), newln);
end of proc two
!MAIN:
    var $u: std::ls[int.sign.32] = [1, 2];
    two(lend.mut $u, lend $u);
!end of MAIN
