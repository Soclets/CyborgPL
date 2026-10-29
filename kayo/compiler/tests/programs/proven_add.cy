# A value from outside, guarded until Precog can prove the addition cannot
# leave an i32 — so the binary carries the plain instruction with the fact
# attached, and no check. Not foldable: nobody knows what getpid returns.
%extern.C proc getpid [] -> i32;
%extern.C proc exit [let $code: i32];

MAIN:
    no_check let $n := getpid();
    if $n > 0:
        if $n < 1000:
            let $x: i32 := $n + 1;
            no_check my.exit(0);
        ;if
    ;if
    no_check my.exit(0);
;MAIN
