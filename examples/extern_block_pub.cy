# Lines in an `extern.C { … }` block may each say `pub` (owner, 2026-10-02: "allow mixing, but block
# overrides"): here `strlen` is public and `write` stays private behind a safe wrapper, as good
# practice #5 says. A `pub` before the block would make every line public.


module text {
    extern.C {
        pub fn strlen [$s: ptr[u8]] -> u64;
        fn write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;
    }
    pub fn say [lent $msg: str.utf8] {
        no_check let $n := write(1, lend $msg, std::str.bytes(lend $msg));
    }
}

MAIN {
    let $word := "hello";
    no_check let $n := text.strlen(lend $word);
    std::print.useful("strlen: ", $n, newln);
    let $line := "written by the wrapper" concat newln;
    text.say(lend $line);
}
