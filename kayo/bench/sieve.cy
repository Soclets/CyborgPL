# The primes up to N, by the sieve of Eratosthenes; the exit code is their
# count's low byte. N comes from the environment and must be 0 to
# 200_000_000; anything else ends the program with exit code 2, as does no N at all.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [let $s: ptr[u8]] -> i64[min: 0, max: 200_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[u8]] -> i64[min: 0, max: 200_000_000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n

proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    # n + 1 falses, as C's calloc gives them (`list.fill`, owner 2026-09-30).
    var $comp: std::ls[bool] = std::ls.fill(value: false, count: $n + 1);
    var $count: i64 = 0;
    var $p: i64 = 2;
    while $p <= $n:
        if not $comp[pos: to.u64($p + 1)]:
            change $count to $count + 1;
            var $m: i64 = $p x $p;
            while $m <= $n:
                change $comp[pos: to.u64($m + 1)] to true;
                change $m to $m + $p;
            ;while
        ;if
        change $p to $p + 1;
    ;while
    ret to.i32($count mod.floor 256);
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
