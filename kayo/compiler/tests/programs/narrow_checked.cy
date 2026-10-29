# Not in expected.txt: what it exits with depends on who runs it. The test
# reads its IR — an unproven narrowing has to reach the binary as a check.
%extern.C proc exit [$code: i32];
%extern.C proc getuid [] -> u32;
MAIN:
    no_check let $u := getuid();
    let $b: u8 := $u;
    no_check my.exit(to.i32($b));
;MAIN
