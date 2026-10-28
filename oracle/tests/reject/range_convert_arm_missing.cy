MAIN:
    let $n := to.i32[min: 0, max: 100]("42")-
    otherwise:
        case not_a_number:
            std::exit(code: 1);
        ;case not_a_number
    ;otherwise;
;MAIN
