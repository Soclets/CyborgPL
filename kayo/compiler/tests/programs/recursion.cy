%extern.C proc exit [$code: i32];

fn fib [$n: i32] -> i32:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib

MAIN:
    no_check my.exit(fib(10));                            # 55
;MAIN
