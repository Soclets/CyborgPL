# Declaring operators: a symbol one, `<+>`, and a word one, `dot`.
record Vec:
    x: float.binary.64_bit; y: float.binary.64_bit;
end of record Vec

# a glyph operator: any run of  not % & * plus minus / < > = ? @ ^ | ~
operator <+> [$a: Vec, $b: Vec] -> Vec:
    return Vec[x = $a.x plus $b.x, y = $a.y plus $b.y];
end of operator <+>

# a word operator, like times / pow / mod.floor
operator dot [$a: Vec, $b: Vec] -> float.binary.64_bit:
    return $a.x times $b.x plus $a.y times $b.y;
end of operator dot

# operands are lent unless the op says give; then the caller says give too
operator <*> [give $a: Vec, $b: float.binary.64_bit] -> Vec:
    change $a.x = $a.x times $b;
    change $a.y = $a.y times $b;
    return give $a;
end of operator <*>

!MAIN:
    let $u := Vec[x = 1, y = 2];
    let $v := Vec[x = 3, y = 4];
    let $w := $u <+> $v;                      # 'u' and 'v' both still here
    std::print.shortest($w.x, " ", $w.y, newline);
    std::print.shortest($u dot $v, newline);
    let $d := ($u <+> $v) dot $v;            # custom operators have no precedence: parenthesise
    std::print.shortest($d, newline);
    let $s := give $u <*> 10;                   # 'u' is gone after this
    std::print.shortest($s.x, " ", $s.y, newline);
!end of MAIN
