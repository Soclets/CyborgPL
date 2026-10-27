# A kit's operators may carry its path or not (owner, 2026-10-22: "It's always optional"), so std's word operators
# may be written with `std::`, as `std::pow` has since 2026-10-02.

!MAIN:
    let $a: integer.signed.32_bit = 2 std::plus 3 std::times 4;
    let $b: integer.signed.32_bit = 7 std::divide.floor 2;
    let $c: integer.unsigned.32_bit = 1 std::shift_left 3;
    std::print.shortest($a, " ", $b, " ", $c, newline);
!end of MAIN
