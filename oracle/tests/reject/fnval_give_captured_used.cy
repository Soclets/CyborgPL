MAIN:
    let $s := "text";
    let $f := fn [] -> i32 with [give $s]:
        ret std::str.bytes(lend $s);
    ;fn;
    std::print.stored($s);
;MAIN
