# Declaring operators: a symbol one, `<+>`, and a word one, `dot`.
struct Vec { x: f64, y: f64 }

# a glyph operator: any run of  not % & * plus minus / < > = ? @ ^ | ~
op <+> ['a': Vec, 'b': Vec] -> Vec {
    return Vec[x = 'a'.x + 'b'.x, y = 'a'.y + 'b'.y];
}

# a word operator, like times / pow / mod.floor
op dot ['a': Vec, 'b': Vec] -> f64 {
    return 'a'.x x 'b'.x + 'a'.y x 'b'.y;
}

# operands are lent unless the op says give; then the caller says give too
op <*> [give 'a': Vec, 'b': f64] -> Vec {
    change 'a'.x = 'a'.x x 'b';
    change 'a'.y = 'a'.y x 'b';
    return give 'a';
}

MAIN {
    let 'u' := Vec[x = 1, y = 2];
    let 'v' := Vec[x = 3, y = 4];
    let 'w' := 'u' <+> 'v';                      # 'u' and 'v' both still here
    std::print.useful['w'.x, " ", 'w'.y, newln];
    std::print.useful['u' dot 'v', newln];
    let 'd' := ('u' <+> 'v') dot 'v';            # custom operators have no precedence: parenthesise
    std::print.useful['d', newln];
    let 's' := give 'u' <*> 10;                   # 'u' is gone after this
    std::print.useful['s'.x, " ", 's'.y, newln];
}
