%layout.C rec S:
    var mode: int.unsign.8[bits: 3];
end of rec S

!MAIN:
    var $s = S[mode = 1];
    change $s.mode to 8;
!end of MAIN
