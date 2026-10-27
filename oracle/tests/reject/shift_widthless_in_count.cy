# a shift's count gives nothing inside it a width (owner, 2026-10-17: "(a)")
!MAIN:
    let $r: integer.signed.64_bit = to.integer.signed.64_bit(1) shift_left (5 shift_right to.integer.unsigned.8_bit(1));
    std::print.shortest($r);
!end of MAIN
