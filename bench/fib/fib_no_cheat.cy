# The same program with `no_cheat` in front of the work (owner, 2026-10-18/19): every call and loop under
# it really runs, so kayo may not fold it, drop it, compute look-alike calls once, or use Precog's tables.
# For the run-time bench's like-for-like rows (owner: "run a fresh one, with cheat and no_cheat").
# The Nth Fibonacci number, recursively; the exit code is its low byte. N
# comes from the environment and must be 0 to 92 — fib(93) does not fit an
# i64 — and anything else ends the program with exit code 2, as does no N at all.
%external.C procedure exit [$code: integer.signed.32_bit];
%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
%external.C procedure atoll [$s: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 92];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 92] or fail[missing, out_of_range]:
    if pointer.is_null($text):
        fail missing;
    end of if
    !no_check let $n := atoll($text) otherwise fail;
    return $n;
end of procedure read_n

function fib [$n: integer.signed.64_bit[minimum: 0, maximum: 92]] -> integer.signed.64_bit:
    if $n less_than 2:
        return $n;
    end of if
    return fib($n minus 1) plus fib($n minus 2);
end of function fib

procedure run [] -> integer.signed.32_bit or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    return to.integer.signed.32_bit(fib($n) modulo.floor 256);
end of procedure run

!MAIN:
    no_skip let $code := run() otherwise 2;
    !no_check my.exit($code);
!end of MAIN
