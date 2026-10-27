# The same program with `no_cheat` in front of the work (owner, 2026-10-18/19): every call and loop under
# it really runs, so kayo may not fold it, drop it, compute look-alike calls once, or use Precog's tables.
# For the run-time bench's like-for-like rows (owner: "run a fresh one, with cheat and no_cheat").
# The primes up to N, by the sieve of Eratosthenes; the exit code is their
# count's low byte. N comes from the environment and must be 0 to
# 200_000_000; anything else ends the program with exit code 2, as does no N at all.
%external.C procedure exit [$code: integer.signed.32_bit];
%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
%external.C procedure atoll [$s: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 200_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 200_000_000] or fail[missing, out_of_range]:
    if pointer.is_null($text):
        fail missing;
    end of if
    !no_check let $n := atoll($text) otherwise fail;
    return $n;
end of procedure read_n

procedure run [] -> integer.signed.32_bit or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    # n + 1 falses, as C's calloc gives them (`list.fill`, owner 2026-09-30).
    variable $comp: std::list[boolean] = std::list.fill(value: false, count: $n plus 1);
    variable $count: integer.signed.64_bit = 0;
    for $p: integer.signed.64_bit [from: 2, to: $n, by: 1]:
        if not $comp[position: to.integer.unsigned.64_bit($p plus 1)]:
            change $count = $count plus 1;
            for $m: integer.signed.64_bit [from: $p times $p, to: $n, by: $p]:
                change $comp[position: to.integer.unsigned.64_bit($m plus 1)] = true;
            end of for
        end of if
    end of for
    return to.integer.signed.32_bit($count modulo.floor 256);
end of procedure run

!MAIN:
    no_skip let $code := run() otherwise 2;
    !no_check my.exit($code);
!end of MAIN
