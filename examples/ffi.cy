# The door to the OS (design/ffi.md). The oracle cannot link C; it emulates
# write, exit, getenv, strlen, malloc, free so this runs.
extern.C fn write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;
extern.C fn strlen [$s: ptr[u8]] -> u64;
extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
extern.C fn malloc [$bytes: u64] -> ptr.mut[u8];
extern.C fn free [$p: ptr.mut[u8]];

layout.C struct Timeval {
    sec: i64,
    usec: i64
}

fn say [lent $text: str.utf8] -> fail[io] {
    no_check let $n := write(1, lend $text, strlen(lend $text));   # a lend is the pointer; the call is a no_check site
    if $n < 0 { fail io; }
}

MAIN {
    say("hello from write" concat newln) otherwise std::exit(1);
    say("hello again" concat newln) otherwise std::exit(1);

    let $key := "HOME";                            # the lent text ends where C expects a 0
    no_check let $home := getenv(lend $key);
    if ptr.is_null($home) { std::print.stored("no HOME", newln); }
    else { no_check if strlen($home) > 0 { std::print.stored("HOME is set", newln); } }

    no_check let $buf := malloc(4);                 # bytes nobody owns
    no_check ptr.write($buf, value: 72);
    no_check ptr.write(ptr.at($buf, by: 1), value: 105);
    no_check std::print.useful(ptr.read($buf), " ", ptr.read(ptr.at($buf, by: 1)), newln);
    no_check free($buf);

    std::print.useful(size.of[i32], " ", size.of[Timeval], " ", align.of[i64], " ", size.of[ptr[u8]], newln);
}
