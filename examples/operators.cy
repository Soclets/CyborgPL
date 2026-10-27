# Declaring operators: a symbol one, `<+>`, and a word one, `dot`.
rec Vec:
    var x: float.bin.64;
    var y: float.bin.64;
end of rec Vec

# a glyph operator: any run of  not % & * plus minus / < > = ? @ ^ | ~
op <+> [lent $a: Vec, lent $b: Vec] -> Vec:
    ret Vec[x = $a.x plus $b.x, y = $a.y plus $b.y];
end of op <+>

# a word operator, like times / pow / mod.floor
op dot [lent $a: Vec, lent $b: Vec] -> float.bin.64:
    ret $a.x times $b.x plus $a.y times $b.y;
end of op dot

# an operand says how it comes, as a function's parameter does: `lent`, or its own (`var` here, since it changes),
# and then the caller says give
op <*> [var $a: Vec, lent $b: float.bin.64] -> Vec:
    change $a.x to $a.x times $b;
    change $a.y to $a.y times $b;
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
