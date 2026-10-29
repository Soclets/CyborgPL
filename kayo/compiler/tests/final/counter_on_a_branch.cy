# A counter stepped on only some trips does not bound how many trips there
# are: for an odd N this loop goes round forever, and the sum overflows.
# The final build must not bound 'acc' by the counter.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [let $s: ptr[u8]] -> i64[min: 0, max: 1000];
proc read_n [let $text: ptr[u8]] -> i64[min: 0, max: 1000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n
proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    var $acc: i64 = 0;
    var $i: i64 = 0;
    while $i <= $n:
        if ($n mod.floor 2) == 0:
            change $i to $i + 1;
        ;if
        change $acc to $acc + 1_000_000_000_000;
    ;while
    ret to.i32($acc mod.floor 256);
;proc run
MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
