# A name given to a failure, `fail sold_out[give $kind]`, reaches its handler whole (design claude's night_trains,
# 2026-10-25): the scope's drop came before the failure gave it away, and the handler read freed memory. The
# text is made when the program runs, and texts made after it would take its freed memory: exit 13 if it is whole.
fn kind_of [$n: i32] -> str.utf8 { return "sleeper for " concat to.stored.str.utf8($n); }
fn take [$n: i32] -> i32 or fail[sold_out: str.utf8] {
    let $kind := kind_of($n);
    if $n equal_to 1 { fail sold_out[give $kind]; }
    return 0;
}
MAIN {
    let $got := take(1) otherwise {
        sold_out $what {
            let $other := kind_of(7);
            let $more := kind_of(8);
            if $what equal_to "sleeper for 1" and $other equal_to "sleeper for 7" and $more equal_to "sleeper for 8" { std::exit(code: 13); }
            std::exit(code: 2);
        }
    };
    std::exit(code: $got);
}
