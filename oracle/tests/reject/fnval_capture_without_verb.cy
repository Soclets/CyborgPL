MAIN:
    let $limit := 60;
    let $f := fn [$s: i32] -> bool with [$limit]:
        ret true;
    ;fn;
;MAIN
