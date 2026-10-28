# The same program with `no_cheat` in front of the work (owner, 2026-10-18/19): every call and loop under
# it really runs, so kayo may not fold it, drop it, compute look-alike calls once, or use Precog's tables.
# For the run-time bench's like-for-like rows (owner: "run a fresh one, with cheat and no_cheat").
# The Nth Fibonacci number, recursively; the exit code is its low byte. N
# comes from the environment and must be 0 to 92 — fib(93) does not fit an
# i64 — and anything else ends the program with exit code 2, as does no N at all.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoll [let $s: ptr[u8]] -> i64[min: 0, max: 92];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[u8]] -> i64[min: 0, max: 92] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    ;if
    no_check let $n := atoll($text)?;
    ret $n;
;proc read_n

fn fib [let $n: i64[min: 0, max: 92]] -> i64:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib

proc run [] -> i32 or fail[missing, out_of_range]:
    let $key := "N";
    no_check let $n := read_n(getenv(lend $key))?;
    ret to.i32(fib($n) mod.floor 256);
;proc run

MAIN:
    no_skip let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
