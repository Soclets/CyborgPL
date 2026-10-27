# Lines in an `extern.C { … }` block may each say `pub` (owner, 2026-10-02: "allow mixing, but block
# overrides"): here `strlen` is public and `write` stays private behind a safe wrapper, as good
# practice #5 says. A `pub` before the block would make every line public.


module text:
    %extern.C:
        pub proc strlen [$s: ptr[int.unsign.8]] -> int.unsign.64;
        proc write [$fd: int.sign.32, $buf: ptr[int.unsign.8], $len: int.unsign.64] -> int.sign.64;
    end of %extern.C
    pub proc say [lent $msg: str.utf8]:
        !no_check let $n := write(1, lend $msg, std::str.bytes(lend $msg));
    end of proc say
end of module text

!MAIN:
    let $word := "hello";
    !no_check let $n := text.strlen(lend $word);
    std::print.shortest("strlen: ", $n, newln);
    let $line := "written by the wrapper" concat newln;
    text.say(lend $line);
!end of MAIN
