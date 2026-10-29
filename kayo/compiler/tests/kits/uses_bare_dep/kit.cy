%extern.C proc exit [let $code: i32];
MAIN:
    no_check my.exit(bare_dep::sum(3, 4) + 1);
;MAIN
