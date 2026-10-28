# In parallel: each prime's multiples crossed off by a `for para` loop, then the count: a `for para`
# reduction.
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
    # cross off the multiples of each prime up to √n, each prime's multiples spread across the cores
    for $p: i64 [from: 2, to: $n, by: 1]:
        if not $comp[pos: to.u64($p + 1)]:
            for %para $m: i64 [from: $p x $p, to: $n, by: $p]:
                change $comp[pos: to.u64($m + 1)] to true;
            ;for
        ;if
    ;for
    # then count: what is left, a reduction across the cores
    var $count: i64 = 0;
    for %para $i: i64 [from: 2, to: $n, by: 1]:
        if not $comp[pos: to.u64($i + 1)]:
            change $count += 1;
        ;if
    ;for
    ret to.i32($count mod.floor 256);
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
