# In parallel: the same loop, forced across every core with `for para` (a reduction into 'acc').
# The sum of i x i mod 7 for i = 1 to N, then mod 1_000_000_007; the exit
# code is the sum's low byte. N comes from the environment and must be 0 to
# 1_000_000_000: `atoll`'s answer is checked at the door, and anything else
# ends the program with exit code 2, as does no N at all.
%external.C procedure exit [$code: int32];
%external.C procedure getenv [$name: pointer[uint8]] -> pointer[uint8];
%external.C procedure atoll [$s: pointer[uint8]] -> int64[min 0, max 1_000_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[uint8]] -> int64[min 0, max 1_000_000_000] or fail[missing, out_of_range]:
    if pointer.is_null[$text]:
        fail missing;
    end of if
    !no_check let $n := atoll[$text] otherwise fail;
    return $n;
end of procedure read_n

function run [] -> int32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n[getenv[lend $key]] otherwise fail;
    variable $acc: int64 = 0;
    for %parallel $i: int64 from 1 to $n by 1:
        change $acc = $acc plus (($i times $i) modulo.floor 7);
    end of for
    return to.int32[($acc modulo.floor 1_000_000_007) modulo.floor 256];
end of function run

!MAIN:
    let $code := run[] otherwise 2;
    !no_check my.exit[$code];
!end of MAIN
