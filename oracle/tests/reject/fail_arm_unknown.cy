fn f [] -> i32 or fail[a]:
    fail a;
;fn f
MAIN:
    let $x := f()-
    otherwise:
        case a:
            std::exit(code: 1);
        ;case a
        case z:
            std::exit(code: 1);
        ;case z
    ;otherwise;
;MAIN
