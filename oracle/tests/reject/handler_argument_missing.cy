fn half [let $n: i32] -> i32 or fail[odd, negative: i32]:
    if $n < 0:
        fail negative[$n];
    ;if
    if ($n div.floor 2) x 2 != $n:
        fail odd;
    ;if
    ret $n div.floor 2;
;fn half

coll.handler half_failed:
    member half_failed.or_default [let $default: i32, let $label: str.utf8]:
        case odd:
            std::println($label, ": odd, using ", $default); ret $default;
        ;case odd
        case negative $v:
            std::println($label, ": ", $v, " is negative, using ", $default); ret $default;
        ;case negative
    ;member half_failed.or_default
;coll.handler half_failed

fn half_or [let $n: i32, let $d: i32] -> i32:
    let $h := half($n) otherwise half_failed.*or_default(default: $d);
    ret $h;
;fn half_or

MAIN:
    std::println(half_or(8, 0), " ", half_or(7, 100), " ", half_or(-4, 5 x 2));
;MAIN
