union.tag Age:
    adult;
    child;
;union.tag Age
fn show [let $a: Age] -> i32:
    ret 1;
;fn show
MAIN:
    let $a := Age.child;
    let $n := show(copy $a);
;MAIN
