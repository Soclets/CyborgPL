%extern.C proc exit [let $code: i32];

fn grade [let $n: i32] -> i32:
    if $n > 90:
        ret 1;
    else if $n > 50:
        ret 2;
    else:
        ret 3;
    ;if
;fn grade

MAIN:
    let $a: i32 := grade(95);
    let $b: i32 := grade(60);
    let $c: i32 := grade(10);
    no_check my.exit(($a x 100) + ($b x 10) + $c);     # 123
;MAIN
