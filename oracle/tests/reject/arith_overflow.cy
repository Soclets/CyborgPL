fn big [$a: i32] -> i32:
    ret $a x $a;
;fn big
MAIN:
    let $r := big(100000);
;MAIN
