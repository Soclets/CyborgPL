# The door to the OS (design/ffi.md). The oracle cannot link C; it emulates
# write, exit, getenv, strlen, malloc, free so this runs.
%external.C procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;
%external.C procedure strlen [$s: pointer[integer.unsigned.8_bit]] -> integer.unsigned.64_bit;
%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
%external.C procedure free [$p: pointer.mutable[integer.unsigned.8_bit]];

%layout.C record Timeval:
    sec: integer.signed.64_bit,
    usec: integer.signed.64_bit
end of layout

procedure say [lent $text: string.utf8] -> fail[io]:
    !no_check let $n := write(1, lend $text, strlen(lend $text));   # a lend is the pointer; the call is a no_check site
    if $n less_than 0:
        fail io;
    end of if
end of procedure say

!MAIN:
    say("hello from write" concatenate newline) otherwise std::exit(code: 1);
    say("hello again" concatenate newline) otherwise std::exit(code: 1);

    let $key := "HOME";                            # the lent text ends where C expects a 0
    !no_check let $home := getenv(lend $key);
    if pointer.is_null($home):
        std::print.stored("no HOME", newline);
    end of if
    else:
        !no_check if strlen($home) greater_than 0:
            std::print.stored("HOME is set", newline);
        end of if
    end of else

    !no_check let $buf := malloc(4);                 # bytes nobody owns
    !no_check pointer.write($buf, value: 72);
    !no_check pointer.write(pointer.at($buf, by: 1), value: 105);
    !no_check std::print.shortest(pointer.read($buf), " ", pointer.read(pointer.at($buf, by: 1)), newline);
    !no_check free($buf);

    std::print.shortest(size.of[integer.signed.32_bit], " ", size.of[Timeval], " ", align.of[integer.signed.64_bit], " ", size.of[pointer[integer.unsigned.8_bit]], newline);
!end of MAIN
