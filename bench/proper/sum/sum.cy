# The sum of i x i mod 7 for i = 1 to N, then mod 1_000_000_007; the exit
# code is the sum's low byte. N comes from the environment and must be 0 to
# 1_000_000_000: `atoll`'s answer is checked at the door, and anything else
# ends the program with exit code 2, as does no N at all.
%extern.C proc exit [let $code: int32];
%extern.C proc getenv [let $name: ptr[uint8]] -> ptr[uint8];
%extern.C proc atoll [let $s: ptr[uint8]] -> int64[min 0, max 1_000_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[uint8]] -> int64[min 0, max 1_000_000_000] or fail[missing, out_of_range]:
    if ptr.is_null[$text]:
        fail missing;
    end of if
    !no_check let $n := atoll[$text] otherwise fail;
    ret $n;
end of proc read_n

fn run [] -> int32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n[getenv[lend $key]] otherwise fail;
    var $acc: int64 = 0;
    for $i: int64 from 1 to $n by 1:
        change $acc = $acc plus (($i times $i) mod.floor 7);
    end of for
    ret to.int32[($acc mod.floor 1_000_000_007) mod.floor 256];
end of fn run

!MAIN:
    let $code := run[] otherwise 2;
    !no_check my.exit[$code];
!end of MAIN
