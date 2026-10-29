# A line that fails whenever it runs but may never run is a check with a
# note, not a compile error (owner, 2026-09-23). A process id is never
# above 10^8, so the conversion never runs — but Precog cannot know that.
%extern.C proc getpid [] -> i32;
%extern.C proc exit [$code: i32];
MAIN:
    no_check let $n := getpid();
    let $p: i16 := -5;
    if $n > 100000000:
        let $x: u64 := to.u64($p);
    ;if
    no_check my.exit(0);
;MAIN
