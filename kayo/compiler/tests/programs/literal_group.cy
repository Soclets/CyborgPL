%extern.C proc exit [$code: i32];
MAIN:
    let $u: u64 := 1;
    let $a: u64 := (0 + 10702988103728947458) + $u;
    let $ok: i32 := 0;
    if $a == 10702988103728947459:
        no_check my.exit(9);
    ;if
    no_check my.exit($ok);
;MAIN
