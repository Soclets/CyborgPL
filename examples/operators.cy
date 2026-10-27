# Declaring operators: a symbol one, `<+>`, and a word one, `dot`.
rec Vec:
    x: float.bin.64;
    y: float.bin.64;
end of rec Vec

# a glyph operator: any run of  not % & * plus minus / < > = ? @ ^ | ~
op <+> [$a: Vec, $b: Vec] -> Vec:
    ret Vec[x = $a.x plus $b.x, y = $a.y plus $b.y];
end of op <+>

# a word operator, like times / pow / mod.floor
op dot [$a: Vec, $b: Vec] -> float.bin.64:
    ret $a.x times $b.x plus $a.y times $b.y;
end of op dot

# operands are lent unless the op says give; then the caller says give too
op <*> [give $a: Vec, $b: float.bin.64] -> Vec:
    change $a.x = $a.x times $b;
    change $a.y = $a.y times $b;
    ret give $a;
end of op <*>

!MAIN:
    let $u := Vec[x = 1, y = 2];
    let $v := Vec[x = 3, y = 4];
    let $w := $u <+> $v;                      # 'u' and 'v' both still here
    std::print.shortest($w.x, " ", $w.y, newln);
    std::print.shortest($u dot $v, newln);
    let $d := ($u <+> $v) dot $v;            # custom operators have no precedence: parenthesise
    std::print.shortest($d, newln);
    let $s := give $u <*> 10;                   # 'u' is gone after this
    std::print.shortest($s.x, " ", $s.y, newln);
!end of MAIN
