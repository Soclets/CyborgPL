# A list of N numbers (i mod 1000 for i = 1 to N), then summed ten times;
# the exit code is the sum's low byte. N comes from the environment and must
# be 0 to 100_000_000; anything else ends the program with exit code 2.
%external.C procedure exit [$code: integer.signed.32_bit];
%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
%external.C procedure atoll [$s: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 100_000_000];

# N, from the text in the environment: missing when there is none, and out
# of range when it is not a number this program takes.
procedure read_n [$text: pointer[integer.unsigned.8_bit]] -> integer.signed.64_bit[minimum: 0, maximum: 100_000_000] or fail[missing, out_of_range]:
    if pointer.is_null($text):
        fail missing;
    end of if
    !no_check let $n := atoll($text) otherwise fail;
    return $n;
end of procedure read_n

procedure run [] -> integer.signed.32_bit or fail[missing, out_of_range]:
    let $key := "N";
    !no_check let $n := read_n(getenv(lend $key)) otherwise fail;
    variable $xs: std::list[integer.signed.64_bit] = [];
    for $i: integer.signed.64_bit [from: 1, to: $n, by: 1]:
        std::list.append(lend.mutable $xs, value: $i modulo.floor 1000);
    end of for
    variable $acc: integer.signed.64_bit = 0;
    for $r: integer.signed.64_bit [from: 1, to: 10, by: 1]:
        for $x in lend $xs:
            change $acc = $acc plus $x;
        end of for
    end of for
    return to.integer.signed.32_bit($acc modulo.floor 256);
end of procedure run

!MAIN:
    let $code := run() otherwise 2;
    !no_check my.exit($code);
!end of MAIN
