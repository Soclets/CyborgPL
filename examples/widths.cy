# A shift's width comes from any operand whose type is known — a name, an expression,
# or the type it is being stored into (decided 2026-09-23). Same for not.bit and the wrapping forms.
!MAIN:
    let $v: int.unsign.32 = 3;
    let $a: int.unsign.32 = 2 shl 3;
    let $b: int.unsign.32 = ($v mod.floor 16) shl 2;
    let $c: int.unsign.32 = not.bit ($v plus 1);
    let $d: int.unsign.32 = ($v plus 1) plus.wrap 5;
    std::print.shortest($a, " ", $b, " ", $c, " ", $d, newln);
!end of MAIN
