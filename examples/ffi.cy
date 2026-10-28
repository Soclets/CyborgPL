# The door to the OS (design/ffi.md). The oracle cannot link C; it emulates
# write, exit, getenv, strlen, malloc, free so this runs.
%extern.C proc write [let $fd: i32, let $buf: ptr[u8], let $len: u64] -> i64;
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc malloc [let $bytes: u64] -> ptr.mut[u8];
%extern.C proc free [let $p: ptr.mut[u8]];

%layout.C rec Timeval:
    var sec: i64;
    var usec: i64;
;%layout.C rec Timeval

proc say [lent $text: str.utf8] -> fail[io]:
    no_check let $n := write(1, lend $text, strlen(lend $text));   # a lend is the pointer; the call is a no_check site
    if $n < 0:
        fail io;
    ;if
;proc say

MAIN:
    say("hello from write" concat newln) ?? std::exit(code: 1);
    say("hello again" concat newln) ?? std::exit(code: 1);

    let $key := "HOME";                            # the lent text ends where C expects a 0
    no_check let $home := getenv(lend $key);
    if ptr.is_null($home):
        std::print.stored("no HOME", newln);
    else:
        no_check if strlen($home) > 0:
            std::print.stored("HOME is set", newln);
        ;if
    ;if

    no_check let $buf := malloc(4);                 # bytes nobody owns
    no_check ptr.write($buf, value: 72);
    no_check ptr.write(ptr.at($buf, by: 1), value: 105);
    no_check std::print(ptr.read($buf), " ", ptr.read(ptr.at($buf, by: 1)), newln);
    no_check free($buf);

    std::print(size.of[i32], " ", size.of[Timeval], " ", align.of[i64], " ", size.of[ptr[u8]], newln);
;MAIN
