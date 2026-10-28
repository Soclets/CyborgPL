MAIN:
    let $limit := 60;
    let $f := fn [let $s: i32] -> bool:
        ret $s >= $limit;
    ;fn;
    let $b := $f(70);
;MAIN
