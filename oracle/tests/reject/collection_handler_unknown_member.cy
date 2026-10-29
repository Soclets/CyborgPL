fn half [$n: i32] -> i32 or fail[odd, negative: i32]:
    if $n < 0:
        fail negative[$n];
    ;if
    ret $n;
;fn half
coll.handler h:
    member h.zero:
        case odd:
            ret 0;
        ;case odd
        case negative $v:
            ret 0;
        ;case negative
    ;member h.zero
;coll.handler h
fn f [] -> i32:
    let $v := half(3) otherwise h.nope;
    ret $v;
;fn f
MAIN:
;MAIN
