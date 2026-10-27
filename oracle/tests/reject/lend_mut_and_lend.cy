procedure two [lent.mutable $a: std::list[integer.signed.32_bit], lent $b: std::list[integer.signed.32_bit]]:
    std::print.shortest(std::list.length(lend $a) plus std::list.length(lend $b), newline);
end of procedure two
!MAIN:
    variable $u: std::list[integer.signed.32_bit] = [1, 2];
    two(lend.mutable $u, lend $u);
!end of MAIN
