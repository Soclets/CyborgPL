# Powers at run time: by squaring, each multiply checked only when the
# answer needs it; xx~ wraps at its values' size. Each true case adds a bit.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    let $s := "ab";
    no_check let $two: i64 := to.i64(strlen(lend $s));
    var $got: i64 = 0;
    if $two std::pow 10 == 1024:
        change $got to $got + 1;
    ;if
    if ($two + 1) std::pow 0 == 1:
        change $got to $got + 2;
    ;if
    if (0 - $two) std::pow 3 == -8:
        change $got to $got + 4;
    ;if
    let $w: u8 := to.u8($two + 1);
    let $e5: u8 := to.u8($two + 3);
    if $w std::pow.wrap $e5 == 243:
        change $got to $got + 8;
    ;if
    let $e6: u8 := to.u8($two + 4);
    if $w std::pow.wrap $e6 == 217:
        change $got to $got + 16;
    ;if
    no_check my.exit(to.i32($got));
;MAIN
