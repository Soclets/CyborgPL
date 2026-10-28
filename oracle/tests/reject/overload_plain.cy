fn f [let $a: i64] -> i64:
    ret 1;
;fn f
fn f [let $b: f64] -> i64:
    ret 2;
;fn f
MAIN:
    std::println(f(1));
;MAIN
