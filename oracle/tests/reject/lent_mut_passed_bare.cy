procedure add1 [lent.mutable $out: std::list[integer.signed.32_bit]]:
    std::list.append(lend.mutable $out, value: 1);
end of procedure add1
procedure walk [lent.mutable $out: std::list[integer.signed.32_bit]]:
    add1($out); std::list.append(lend.mutable $out, value: 2);
end of procedure walk
!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = []; walk(lend.mutable $xs); std::print.shortest(std::list.length(lend $xs), newline);
!end of MAIN
