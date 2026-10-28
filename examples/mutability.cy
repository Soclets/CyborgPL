# Plain, `mut` and `immut` variables, and an `immut` parameter.
fn twice [let $n: i32] -> i32:
    ret $n x 2;
;fn twice

MAIN:
    var $plain = 1;
    change $plain += 1;
    var $counter = 0;
    change $counter += 1;
    let $limit: i32 := 10;
    std::println($plain, " ", $counter, " ", $limit, " ", twice($limit));
;MAIN
