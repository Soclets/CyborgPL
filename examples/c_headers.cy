# C headers (owner, 2026-10-10): `incl.C.system["…"]` reads the system's headers while building, so C's
# functions need no hand-written `extern.C`. A header brings its functions, globals and plain `#define`
# constants (`$BUFSIZ`); they are called bare where the line is, or with a module's prefix when the
# line sits in a `module` block (`pub.kit` there, as for any item a module shares). A call into C
# is a `no_check` site, as for `extern.C`. The oracle runs only the C functions it imitates (write,
# strlen, getenv, malloc, free, exit, qsort).
include.%C.system["unistd.h", "stdio.h"];

module text:
    public.kit include.%C.system["string.h"];
end of module text

!MAIN:
    let $msg := "hello through C's write";
    !no_check let $n := write(1, lend $msg, text.strlen(lend $msg));
    std::print.shortest(newline, $n, " bytes; BUFSIZ ", $BUFSIZ, ", EOF ", $EOF, ", SEEK_END ", $SEEK_END, newline);
!end of MAIN
