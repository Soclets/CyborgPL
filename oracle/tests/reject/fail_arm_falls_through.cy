fn f [] -> i32 or fail[a]:
    fail a;
;fn f
MAIN:
    let $x := f()-
    otherwise:
        case a:
            std::print.stored("hm");
        ;case a
    ;otherwise;
;MAIN
