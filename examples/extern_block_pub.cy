# Lines in an `extern.C { … }` block may each say `pub` (owner, 2026-10-02: "allow mixing, but block
# overrides"): here `strlen` is public and `write` stays private behind a safe wrapper, as good
# practice #5 says. A `pub` before the block would make every line public.


module text:
    %extern.C:
        pub proc strlen [let $s: ptr[u8]] -> u64;
        proc write [let $fd: i32, let $buf: ptr[u8], let $len: u64] -> i64;
    ;%extern.C
    pub proc say [lent $msg: str.utf8]:
        no_check let $n := write(1, lend $msg, std::str.bytes(lend $msg));
    ;proc say
;module text

MAIN:
    let $word := "hello";
    no_check let $n := text.strlen(lend $word);
    std::println("strlen: ", $n);
    let $line := "written by the wrapper" ++ newln;
    text.say(lend $line);
;MAIN
