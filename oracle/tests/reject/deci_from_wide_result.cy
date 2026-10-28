fn f [let $a: i64] -> %d64:
    let $b := $a x 1000;
    ret to.%d64($b);
;fn f
MAIN:
    std::print(f(5), newln);
;MAIN
