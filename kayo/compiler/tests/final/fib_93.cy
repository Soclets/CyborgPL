# fib on 0 to 93, N from outside: fib(93) does not fit an i64, so the
# final build must refuse where it lands.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [$s: ptr[u8]] -> i64[min: 0, max: 93];
fn fib [$n: i64[min: 0, max: 93]] -> i64:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib
proc read_n [$text: ptr[u8]] -> i64[min: 0, max: 93] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n
proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    ret to.i32(fib($n) mod.floor 256);
;proc run
MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
