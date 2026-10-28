fn twice [let $n: i64] -> i64:
    ret $n x 2;
;fn twice
MAIN:
    std::print(twice[2], newln);
;MAIN
