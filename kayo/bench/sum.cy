# The sum of i x i mod 7 for i = 1 to N, kept below 1_000_000_007; the exit
# code is the sum's low byte. N comes from the environment and must be 0 to
# 1_000_000_000: `atoll`'s answer is checked at the door, and anything else
# ends the program with exit code 2, as does no N at all.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [$s: ptr[u8]] -> i64[min: 0, max: 1_000_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [$text: ptr[u8]] -> i64[min: 0, max: 1_000_000_000] or fail[missing, out_of_range]:
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
    var $i: i64 = 1;
    while $i <= $n:
        change $acc to ($acc + (($i x $i) mod.floor 7)) mod.floor 1_000_000_007;
        change $i to $i + 1;
    ;while
    ret to.i32($acc mod.floor 256);
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
