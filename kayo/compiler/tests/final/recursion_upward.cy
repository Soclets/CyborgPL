# A function that calls itself on a larger argument is not proven by
# induction on the arguments: that call is to a tuple not yet done, which
# can be anything the return type holds. up(98) is past an i64 in truth.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [let $s: ptr[u8]] -> i64[min: 0, max: 100];
fn up [let $n: i64[min: 0, max: 100]] -> i64:
    if $n >= 100:
        ret 1;
    ;if
    ret up($n + 1) + 9_000_000_000_000_000_000;
;fn up
proc read_n [let $text: ptr[u8]] -> i64[min: 0, max: 100] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n
proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    ret to.i32(up($n) mod.floor 256);
;proc run
MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
