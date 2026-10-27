# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times;
# the exit code is the sum's low byte. N comes from the environment and must
# be 0 to 100_000_000; anything else ends the program with exit code 2.
%extern.C proc exit [let $code: int32];
%extern.C proc getenv [let $name: ptr[uint8]] -> ptr[uint8];
%extern.C proc atoll [let $s: ptr[uint8]] -> int64[min 0, max 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[uint8]] -> int64[min 0, max 100_000_000] or fail[missing, out_of_range]:
    if ptr.is_null[$text]:
        fail missing;
    end of if
    !no_check let $n := atoll[$text] otherwise fail;
    ret $n;
end of proc read_n

fn run [] -> int32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n[getenv[lend $key]] otherwise fail;
    var $xs: ls[int64] := [];
    for $i: int64 from 1 to $n by 1:
        ls.append[lend.mut $xs, value $i mod.floor 1000];
    end of for
    var $acc: int64 := 0;
    for $r: int64 from 1 to 10 by 1:
        for $x in lend $xs:
            change $acc to $acc plus $x;
        end of for
    end of for
    ret to.int32[$acc mod.floor 256];
end of fn run

!MAIN:
    let $code := run[] otherwise 2;
    !no_check my.exit[$code];
!end of MAIN
