%layout.C record S:
    mode: integer.unsigned.8_bit[bits: 3];
end of record S

!MAIN:
    variable $s := S[mode = 1];
    change $s.mode = 8;
!end of MAIN
