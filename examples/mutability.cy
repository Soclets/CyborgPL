# `var` and `let` variables, and a `let` parameter.
fn twice [let $n: i32] -> i32:
    ret $n x 2;
;fn twice

MAIN:
    var $plain = 1;
    change $plain to $plain + 1;
    var $counter = 0;
    change $counter to $counter + 1;
    let $limit: i32 := 10;
    std::print($plain, " ", $counter, " ", $limit, " ", twice($limit), newln);
;MAIN
