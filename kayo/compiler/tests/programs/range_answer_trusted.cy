# The same, but one range in the program rests on `nocheck`'s word: 'w'
# is stored into an i64[min 0, max 1000] unasked. No range is then a fact to
# hand LLVM (owner, 2026-09-29: c), and the answer is not clamped.
%extern.C proc exit [$code: i32];
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
proc pick [] -> i64[min: 0, max: 1000] or fail[missing]:
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p):
        ret 3;
    ;if
    fail missing;
;proc pick
fn widen [$w: i64] -> i64:
    no_check let $r: i64[min: 0, max: 1000] := $w;
    ret $r;
;fn widen
MAIN:
    let $n := pick() ?? 0;
    no_check my.exit(to.i32($n + widen(3)));
;MAIN
