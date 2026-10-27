# The door to the OS (design/ffi.md). The oracle cannot link C; it emulates
# write, exit, getenv, strlen, malloc, free so this runs.
%extern.C proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;
%extern.C proc strlen [let $s: ptr[int.unsign.8]] -> int.unsign.64;
%extern.C proc getenv [let $name: ptr[int.unsign.8]] -> ptr[int.unsign.8];
%extern.C proc malloc [let $bytes: int.unsign.64] -> ptr.mut[int.unsign.8];
%extern.C proc free [let $p: ptr.mut[int.unsign.8]];

%layout.C rec Timeval:
    var sec: int.sign.64;
    var usec: int.sign.64;
end of rec Timeval

proc say [lent $text: str.utf8] -> fail[io]:
    !no_check let $n := write(1, lend $text, strlen(lend $text));   # a lend is the pointer; the call is a no_check site
    if $n lt 0:
        fail io;
    end of if
end of proc say

!MAIN:
    say("hello from write" concat newln) otherwise std::exit(code: 1);
    say("hello again" concat newln) otherwise std::exit(code: 1);

    let $key := "HOME";                            # the lent text ends where C expects a 0
    !no_check let $home := getenv(lend $key);
    if ptr.is_null($home):
        std::print.stored("no HOME", newln);
    end of if
    else:
        !no_check if strlen($home) gt 0:
            std::print.stored("HOME is set", newln);
        end of if
    end of else

    !no_check let $buf := malloc(4);                 # bytes nobody owns
    !no_check ptr.write($buf, value: 72);
    !no_check ptr.write(ptr.at($buf, by: 1), value: 105);
    !no_check std::print.shortest(ptr.read($buf), " ", ptr.read(ptr.at($buf, by: 1)), newln);
    !no_check free($buf);

    std::print.shortest(size.of[int.sign.32], " ", size.of[Timeval], " ", align.of[int.sign.64], " ", size.of[ptr[int.unsign.8]], newln);
!end of MAIN
