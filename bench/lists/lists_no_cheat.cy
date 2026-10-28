# The same program with `no_skip` (was `no_cheat`) in front of the work (owner, 2026-10-18/19): every call and loop under
# it really runs, so kayo may not fold it, drop it, compute look-alike calls once, or use Precog's tables.
# For the run-time bench's like-for-like rows (owner: "run a fresh one, with cheat and no_cheat").
# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times;
# the exit code is the sum's low byte. N comes from the environment and must
# be 0 to 100_000_000; anything else ends the program with exit code 2.
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
    for $i: i64 [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $xs, value: $i mod.floor 1000);
    ;for
    var $acc: i64 = 0;
    for $r: i64 [from: 1, to: 10, by: 1]:
        for $x in lend $xs:
            change $acc to $acc + $x;
        ;for
    ;for
    ret to.i32($acc mod.floor 256);
;proc run

MAIN:
    no_skip let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
