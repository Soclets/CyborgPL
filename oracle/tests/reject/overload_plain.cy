fn f [$a: i64] -> i64:
    ret 1;
;fn f
fn f [$b: f64] -> i64:
    ret 2;
;fn f
MAIN:
    std::print(f(1), newln);
;MAIN
