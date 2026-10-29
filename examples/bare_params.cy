# A name inside brackets is fixed unless it says `var` (owner, 2026-10-29: "(c)"): a parameter or an operand written bare
# never changes, and one that changes says `var`. A declaration on its own line still says `let` or `var`.
fn add [$a: i32, $b: i32] -> i32:
    ret $a + $b;
;fn add

fn countdown [var $n: i32] -> i32:
    change $n to $n - 1;
    ret $n;
;fn countdown

op <+> [$a: i32, $b: i32] -> i32:
    ret $a + $b + 1;
;op <+>

MAIN:
    let $sum := add(2, 3);
    std::print($sum, " ", countdown(5), " ", 2 <+> 3, newln);
;MAIN
