fn f [let $a: i64] -> %d64:
    let $b := $a x 1000;
    ret to.%d64($b);
;fn f
MAIN:
    std::println(f(5));
;MAIN
