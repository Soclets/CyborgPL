# A C union holds one field at a time, and only C knows which: reading a field is `no_check`, writing is always
# safe (owner, 2026-10-19). A bit-field is `bits N` in a `layout.C` field's bracket, holding what N bits hold.
%layout.C union Number:
    i: i32;
    f: f32;
;%layout.C union Number
%layout.C rec Status:
    var mode: u8[bits: 3];
    var ready: bool[bits: 1];
    var level: i8[bits: 4];
;%layout.C rec Status

MAIN:
    var $n = Number[f = 1.0];
    no_check let $raw := $n.i;
    change $n.i to 7;
    no_check let $back := $n.i;
    var $s = Status[mode = 5, ready = true, level = -8];
    change $s.mode to 7;
    std::print($raw, " ", $back, " ", $s.mode, " ", $s.level, newln);
;MAIN
