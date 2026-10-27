# A vector's lane has the lane's type, so a shift of it has a width (check claude, seed 42640), as a
# list item does: 's'[pos 2] is a u64 here.
!MAIN:
    let $s: simd[integer.unsigned.64_bit, count: 2] = simd.fill(value: 40);
    if 3 less_than ($s[position: 2] shift_right 2):
        std::print.shortest("10 is more than 3", newline);
    end of if
!end of MAIN
