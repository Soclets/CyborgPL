# An unproven site keeps its check in a working build, and a check that
# fires stops the program with 101 and says where.
%extern.C proc exit [let $code: i32];
%extern.C proc getpid [] -> i32;

MAIN:
    no_check let $pid := getpid();
    let $huge: i32 := 2_000_000_000;
    let $boom: i32 := $pid x $huge;
    no_check my.exit($boom);
;MAIN
