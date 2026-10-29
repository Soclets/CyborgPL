# A name given to a failure, `fail sold_out[give $kind]`, reaches its handler whole (design claude's night_trains,
# 2026-10-25): the scope's drop came before the failure gave it away, and the handler read freed memory. The
# text is made when the program runs, and texts made after it would take its freed memory: exit 13 if it is whole.
fn kind_of [let $n: i32] -> str.utf8:
    ret "sleeper for " concat to.stored.str.utf8($n);
;fn kind_of
fn take [let $n: i32] -> i32 or fail[sold_out: str.utf8]:
    let $kind := kind_of($n);
    if $n == 1:
        fail sold_out[give $kind];
    ;if
    ret 0;
;fn take
fn try [] -> i32:
    let $got := take(1)-
    otherwise:
        case sold_out $what:
            let $other := kind_of(7);
            let $more := kind_of(8);
            if $what == "sleeper for 1" and $other == "sleeper for 7" and $more == "sleeper for 8":
                ret 13;
            ;if
            ret 2;
        ;case sold_out
    ;otherwise;
    ret $got;
;fn try
MAIN:
    std::exit(code: try());
;MAIN
