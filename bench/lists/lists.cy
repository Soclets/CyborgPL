# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times;
# the exit code is the sum's low byte. N comes from the environment and must
# be 0 to 100_000_000; anything else ends the program with exit code 2.
%extern.C proc exit [let $code: int.sign.32];
%extern.C proc getenv [let $name: ptr[int.unsign.8]] -> ptr[int.unsign.8];
%extern.C proc atoll [let $s: ptr[int.unsign.8]] -> int.sign.64[min: 0, max: 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
proc read_n [let $text: ptr[int.unsign.8]] -> int.sign.64[min: 0, max: 100_000_000] or fail[missing, out_of_range]:
    if ptr.is_null($text):
        fail missing;
    end of if
    !no_check let $n := atoll($text) otherwise fail;
    ret $n;
end of proc read_n

proc run [] -> int.sign.32 or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    var $xs: std::ls[int.sign.64] = [];
    for $i: int.sign.64 [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $xs, value: $i mod.floor 1000);
    end of for
    var $acc: int.sign.64 = 0;
    for $r: int.sign.64 [from: 1, to: 10, by: 1]:
        for $x in lend $xs:
            change $acc to $acc plus $x;
        end of for
    end of for
    ret to.int.sign.32($acc mod.floor 256);
end of proc run

!MAIN:
    let $code := run() otherwise 2;
    !no_check my.exit($code);
!end of MAIN
