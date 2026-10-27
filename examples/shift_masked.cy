# `shl.masked` / `shr.masked` shift as the chip does (owner, 2026-10-07): the count is masked to the
# width (70 on 64 bits is 6) and bits pushed past the width are cut off; no check site. Plain
# `shl` / `shr` stay checked. "masked" over "wrap", which reads as rotating the bits ("Whatever's accurate").
!MAIN:
    let $b: integer.unsigned.8_bit = 255;
    let $x: integer.signed.64_bit = 1;
    let $up: integer.unsigned.8_bit = $b shift_left.masked 1;        # 510 cut to 8 bits: 254
    let $far: integer.signed.64_bit = $x shift_left.masked 70;      # 70 masked to 6: 64
    let $down: integer.unsigned.8_bit = $b shift_right.masked 9;      # 9 masked to 1: 127
    let $neg: integer.unsigned.32_bit = (0 minus 1) shift_right.masked 1;   # -1's bits, then shifted: 2147483647
    std::print.shortest($up, " ", $far, " ", $down, " ", $neg, newline);
!end of MAIN
