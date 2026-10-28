# In parallel: the top of the recursion as four `%spawn.task` tasks, each sending its part on a channel;
# below that, naive recursion as the program asks.
# The Nth Fibonacci number, recursively; the exit code is its low byte. N
# comes from the environment and must be 0 to 92 — fib(93) does not fit an
# int64 — and anything else ends the program with exit code 2, as does no N at all.
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
    if $n < 4:
        ret to.i32(fib($n) mod.floor 256);
    ;if
    # fib(n) = fib(n-2) + 2 fib(n-3) + fib(n-4): four tasks, each on its own channel
    let $c1: %channel[i64] := %channel.empty;
    let $c2: %channel[i64] := %channel.empty;
    let $c3: %channel[i64] := %channel.empty;
    let $c4: %channel[i64] := %channel.empty;
    %spawn.task with [lend $c1, lend $n]:
        %channel.send(lend $c1, value: fib($n - 2));
    ;%spawn.task
    %spawn.task with [lend $c2, lend $n]:
        %channel.send(lend $c2, value: fib($n - 3));
    ;%spawn.task
    %spawn.task with [lend $c3, lend $n]:
        %channel.send(lend $c3, value: fib($n - 3));
    ;%spawn.task
    %spawn.task with [lend $c4, lend $n]:
        %channel.send(lend $c4, value: fib($n - 4));
    ;%spawn.task
    let $a := %channel.take(lend $c1) ?? 0;
    let $b := %channel.take(lend $c2) ?? 0;
    let $c := %channel.take(lend $c3) ?? 0;
    let $d := %channel.take(lend $c4) ?? 0;
    # each part's low byte first: what comes off a channel could be any int64
    ret to.i32((($a mod.floor 256) + ($b mod.floor 256) + ($c mod.floor 256) + ($d mod.floor 256)) mod.floor 256);
;proc run

MAIN:
    let $code := run() ?? 2;
    no_check my.exit($code);
;MAIN
