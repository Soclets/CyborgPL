# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times by
# position; the exit code is the sum's low byte. N comes from the
# environment and must be 0 to 100_000_000; anything else ends the program
# with exit code 2.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [let $s: ptr[u8]] -> i64[min: 0, max: 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[u8]] -> i64[min: 0, max: 100_000_000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n

proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    var $xs: std::ls[i64] = [];
    var $i: i64 = 1;
    while $i <= $n:
        std::ls.append(lend.mut $xs, value: $i mod.floor 1000);
        change $i to $i + 1;
    ;while
    var $acc: i64 = 0;
    var $r: i64 = 0;
    while $r < 10:
        var $j: u64 = 1;
        while $j <= std::ls.len(lend $xs):
            change $acc to $acc + $xs[pos: $j];
            change $j to $j + 1;
        ;while
        change $r to $r + 1;
    ;while
    ret to.i32($acc mod.floor 256);
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
