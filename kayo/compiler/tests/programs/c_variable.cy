# extern let (ffi.md): a C global, read and written as C's own variable.
# getopt's optind starts at 1; written 5, read back 5: 1 x 10 + 5 = 15.
%extern.C proc exit [$code: i32];
%extern.C var $optind: i32;
MAIN:
    no_check let $first := $optind;
    no_check change $optind to 5;
    no_check let $after := $optind;
    no_check my.exit($first x 10 + $after);
;MAIN
