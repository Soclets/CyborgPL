fn twice [let $n: i64] -> i64:
    ret $n x 2;
;fn twice
MAIN:
    std::println(twice[2]);
;MAIN
