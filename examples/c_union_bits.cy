# A C union holds one field at a time, and only C knows which: reading a field is `no_check`, writing is always
# safe (owner, 2026-10-19). A bit-field is `bits N` in a `layout.C` field's bracket, holding what N bits hold.
%layout.C union Number:
    i: integer.signed.32_bit, f: float.binary.32_bit
end of layout
%layout.C record Status:
    mode: integer.unsigned.8_bit[bits: 3], ready: boolean[bits: 1], level: integer.signed.8_bit[bits: 4]
end of layout

!MAIN:
    variable $n := Number[f = 1.0];
    !no_check let $raw := $n.i;
    change $n.i = 7;
    !no_check let $back := $n.i;
    variable $s := Status[mode = 5, ready = true, level = negative 8];
    change $s.mode = 7;
    std::print.shortest($raw, " ", $back, " ", $s.mode, " ", $s.level, newline);
!end of MAIN
