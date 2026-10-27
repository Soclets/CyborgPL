# The Nth Fibonacci number, recursively; the exit code is its low byte. N
# comes from the environment and must be 0 to 92 — fib(93) does not fit an
# int64 — and anything else ends the program with exit code 2, as does no N at all.
%extern.C proc exit [let $code: int32];
%extern.C proc getenv [let $name: ptr[uint8]] -> ptr[uint8];
%extern.C proc atoll [let $s: ptr[uint8]] -> int64[min 0, max 92];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[uint8]] -> int64[min 0, max 92] or fail[missing, out_of_range]:
    if ptr.is_null[$text]:
        fail missing;
    end of if
    !no_check let $n := atoll[$text] otherwise fail;
    ret $n;
end of proc read_n

fn fib [let $n: int64[min 0, max 92]] -> int64:
    if $n less_than 2:
        ret $n;
    end of if
    ret fib[$n minus 1] plus fib[$n minus 2];
end of fn fib

fn run [] -> int32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n[getenv[lend $key]] otherwise fail;
    ret to.int32[fib[$n] mod.floor 256];
end of fn run

!MAIN:
    let $code := run[] otherwise 2;
    !no_check my.exit[$code];
!end of MAIN
