# A list's length is a u64 and an item has the element type, so either gives a shift its width
# (check claude, seeds 5018 and 5054, 2026-10-07).
!MAIN:
    let $xs: std::list[integer.unsigned.8_bit] = [10, 20, 30,];
    std::print.shortest((std::list.length(lend $xs) shift_left 1) greater_than 5, " ", ($xs[position: 1] shift_right 2) equal_to 2, newline);
!end of MAIN
