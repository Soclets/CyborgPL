%extern.C proc exit [let $code: i32];
union.tag Shape:
    dot;
    line: i32;
    box: i32;
;union.tag Shape
fn measure [let $s: Shape] -> i32:
    match $s:
        case dot:
            ret 1;
        ;case dot
        case line $n:
            ret $n + 10;
        ;case line
        case box $n:
            ret $n + 100;
        ;case box
    ;match
;fn measure
MAIN:
    let $a := Shape.dot;
    let $b := Shape.line(5);
    let $c := Shape.box(7);
    no_check my.exit(measure(give $a) + measure(give $b) + measure(give $c));
;MAIN
