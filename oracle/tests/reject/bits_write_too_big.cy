%layout.C rec S:
    var mode: u8[bits: 3];
;%layout.C rec S

MAIN:
    var $s = S[mode = 1];
    change $s.mode to 8;
;MAIN
