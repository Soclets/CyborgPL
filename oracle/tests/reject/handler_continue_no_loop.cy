coll.handler failed:
    member failed.skip:
        case bad:
            continue;
        ;case bad
    ;member failed.skip
;coll.handler failed

fn small [let $n: i32] -> i32 or fail[bad]:
    if $n > 2:
        fail bad;
    ;if
    ret $n;
;fn small

MAIN:
    let $x := small(1) otherwise failed.*skip;
    std::println($x);
;MAIN
