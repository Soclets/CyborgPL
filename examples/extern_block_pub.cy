# Lines in an `extern.C { … }` block may each say `pub` (owner, 2026-10-02: "allow mixing, but block
# overrides"): here `strlen` is public and `write` stays private behind a safe wrapper, as good
# practice #5 says. A `pub` before the block would make every line public.


module text:
    %external.C:
        public procedure strlen [$s: pointer[integer.unsigned.8_bit]] -> integer.unsigned.64_bit;
        procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;
    end of %external.C
    public procedure say [lent $msg: string.utf8]:
        !no_check let $n := write(1, lend $msg, std::string.bytes(lend $msg));
    end of procedure say
end of module text

!MAIN:
    let $word := "hello";
    !no_check let $n := text.strlen(lend $word);
    std::print.shortest("strlen: ", $n, newline);
    let $line := "written by the wrapper" concatenate newline;
    text.say(lend $line);
!end of MAIN
