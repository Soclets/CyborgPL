# A shift's width comes from any operand whose type is known — a name, an expression,
# or the type it is being stored into (decided 2026-09-23). Same for not.bit and the wrapping forms.
MAIN:
    let $v: u32 := 3;
    let $a: u32 := 2 shl 3;
    let $b: u32 := ($v mod.floor 16) shl 2;
    let $c: u32 := not.bit ($v + 1);
    let $d: u32 := ($v + 1) plus.wrap 5;
    std::println($a, " ", $b, " ", $c, " ", $d);
;MAIN
