# The primes up to N, by the sieve of Eratosthenes; the exit code is their
# count's low byte. N comes from the environment and must be 0 to
# 200_000_000; anything else ends the program with exit code 2, as does no N at all.
%external.C procedure exit [$code: int32];
%external.C procedure getenv [$name: pointer[uint8]] -> pointer[uint8];
%external.C procedure atoll [$s: pointer[uint8]] -> int64[min 0, max 200_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[uint8]] -> int64[min 0, max 200_000_000] or fail[missing, out_of_range]:
    if pointer.is_null[$text]:
        fail missing;
    end of if
    !no_check let $n := atoll[$text] otherwise fail;
    return $n;
end of procedure read_n

function run [] -> int32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n[getenv[lend $key]] otherwise fail;
    # n + 1 falses, as C's calloc gives them (`list.fill`, owner 2026-09-30).
    variable $comp: list[boolean] = list.fill[value false, count $n plus 1];
    variable $count: int64 = 0;
    for $p: int64 from 2 to $n by 1:
        if not $comp[pos to.uint64[$p plus 1]]:
            change $count = $count plus 1;
            for $m: int64 from $p times $p to $n by $p:
                change $comp[pos to.uint64[$m plus 1]] = true;
            end of for
        end of if
    end of for
    return to.int32[$count modulo.floor 256];
end of function run

!MAIN:
    let $code := run[] otherwise 2;
    !no_check my.exit[$code];
!end of MAIN
