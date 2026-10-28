fn f [] -> fail[why: i32]:
    fail why["x"];
;fn f
MAIN:
    f()-
    otherwise:
        case why $w:
        ;case why
    ;otherwise;
;MAIN
