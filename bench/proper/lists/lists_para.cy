# In parallel: the list made at its full size, each place filled by a `for para` round, and each of
# the ten sums a `for para` reduction.
# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times;
# the exit code is the sum's low byte. N comes from the environment and must
# be 0 to 100_000_000; anything else ends the program with exit code 2.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [$s: ptr[u8]] -> i64[min: 0, max: 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [$text: ptr[u8]] -> i64[min: 0, max: 100_000_000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n

proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    var $xs: std::ls[i64] = std::ls.fill(value: 0, count: $n);
    for %para $i: i64 [from: 1, to: $n, by: 1]:
        change $xs[pos: to.u64($i)] to $i mod.floor 1000;
    ;for
    var $acc: i64 = 0;
    for $r: i64 [from: 1, to: 10, by: 1]:
        for %para $x in lend $xs:
            change $acc to $acc + $x;
        ;for
    ;for
    ret to.i32($acc mod.floor 256);
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
