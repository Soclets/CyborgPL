# A C union holds one field at a time, and only C knows which: reading a field is `no_check`, writing is always
# safe (owner, 2026-10-19). A bit-field is `bits N` in a `layout.C` field's bracket, holding what N bits hold.
layout.C union Number { i: i32, f: f32 }
layout.C struct Status { mode: u8[bits: 3], ready: bool[bits: 1], level: i8[bits: 4] }

MAIN {
    let mut $n := Number[f = 1.0];
    no_check let $raw := $n.i;
    change $n.i = 7;
    no_check let $back := $n.i;
    let mut $s := Status[mode = 5, ready = true, level = -8];
    change $s.mode = 7;
    std::print.useful($raw, " ", $back, " ", $s.mode, " ", $s.level, newln);
}
