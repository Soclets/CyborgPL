procedure grow [@y] [lent.mutable @y $xs: std::list[integer.signed.32_bit]] -> lent.mutable @y std::list[integer.signed.32_bit]:
    std::list.append(lend.mutable $xs, value: 9); return lend.mutable $xs;
end of procedure grow
procedure two [lent.mutable $a: std::list[integer.signed.32_bit], lent $b: std::list[integer.signed.32_bit]]:
    std::print.shortest(std::list.length(lend $a) plus std::list.length(lend $b), newline);
end of procedure two
!MAIN:
    variable $u: std::list[integer.signed.32_bit] = [1, 2];
    two(grow(lend.mutable $u), lend $u);
!end of MAIN
