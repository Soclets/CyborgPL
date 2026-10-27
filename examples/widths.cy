# A shift's width comes from any operand whose type is known — a name, an expression,
# or the type it is being stored into (decided 2026-09-23). Same for not.bit and the wrapping forms.
!MAIN:
    let $v: integer.unsigned.32_bit = 3;
    let $a: integer.unsigned.32_bit = 2 shift_left 3;
    let $b: integer.unsigned.32_bit = ($v modulo.floor 16) shift_left 2;
    let $c: integer.unsigned.32_bit = not.bit ($v plus 1);
    let $d: integer.unsigned.32_bit = ($v plus 1) plus.wrap 5;
    std::print.shortest($a, " ", $b, " ", $c, " ", $d, newline);
!end of MAIN
