# Declaring operators: a symbol one, `<+>`, and a word one, `dot`.
rec Vec:
    var x: f64;
    var y: f64;
;rec Vec

# a glyph operator: any run of  not % & * plus minus / < > = ? @ ^ | ~
op <+> [lent $a: Vec, lent $b: Vec] -> Vec:
    ret Vec[x = $a.x + $b.x, y = $a.y + $b.y];
;op <+>

# a word operator, like times / pow / mod.floor
op dot [lent $a: Vec, lent $b: Vec] -> f64:
    ret $a.x x $b.x + $a.y x $b.y;
;op dot

# an operand says how it comes, as a function's parameter does: `lent`, or its own (`var` here, since it changes),
# and then the caller says give
op <*> [var $a: Vec, lent $b: f64] -> Vec:
    change $a.x to $a.x x $b;
    change $a.y to $a.y x $b;
    ret give $a;
;op <*>

MAIN:
    let $u := Vec[x = 1, y = 2];
    let $v := Vec[x = 3, y = 4];
    let $w := $u <+> $v;                      # 'u' and 'v' both still here
    std::print($w.x, " ", $w.y, newln);
    std::print($u dot $v, newln);
    let $d := ($u <+> $v) dot $v;            # custom operators have no precedence: parenthesise
    std::print($d, newln);
    let $s := give $u <*> 10;                   # 'u' is gone after this
    std::print($s.x, " ", $s.y, newln);
;MAIN
