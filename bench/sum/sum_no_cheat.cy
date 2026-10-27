# The same program with `no_cheat` in front of the work (owner, 2026-10-18/19): every call and loop under
# it really runs, so kayo may not fold it, drop it, compute look-alike calls once, or use Precog's tables.
# For the run-time bench's like-for-like rows (owner: "run a fresh one, with cheat and no_cheat").
# The sum of i x i mod 7 for i = 1 to N, then mod 1_000_000_007; the exit
# code is the sum's low byte. N comes from the environment and must be 0 to
# 1_000_000_000: `atoll`'s answer is checked at the door, and anything else
# ends the program with exit code 2, as does no N at all.
%extern.C proc exit [let $code: int.sign.32];
%extern.C proc getenv [let $name: ptr[int.unsign.8]] -> ptr[int.unsign.8];
%extern.C proc atoll [let $s: ptr[int.unsign.8]] -> int.sign.64[min: 0, max: 1_000_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[int.unsign.8]] -> int.sign.64[min: 0, max: 1_000_000_000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    end of if
    !no_check let $n := atoll($text) otherwise fail;
    ret $n;
end of proc read_n

proc run [] -> int.sign.32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    var $acc: int.sign.64 := 0;
    for $i: int.sign.64 [from: 1, to: $n, by: 1]:
        change $acc to $acc plus (($i times $i) mod.floor 7);
    end of for
    ret to.int.sign.32(($acc mod.floor 1_000_000_007) mod.floor 256);
end of proc run

!MAIN:
    no_skip let $code := run() otherwise 2;
    !no_check my.exit($code);
!end of MAIN
