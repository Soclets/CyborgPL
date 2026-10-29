fn look [lent $a: str.utf8] -> i32:
    ret 1;
;fn look
fn apply [$f: fn[str.utf8] -> i32] -> i32:
    ret $f("x");
;fn apply
MAIN:
    let $x := apply(look);
;MAIN
