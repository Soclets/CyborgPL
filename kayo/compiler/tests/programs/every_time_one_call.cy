# A site reached from two calls fails "every time" only if it does on both
# (check claude, seed 7194). `double` is called with 1, where `'p' x 2`
# fits, and with 5e18, where it never does; the second call is behind an
# `if` the run does not take, so the program ends 2. It was refused as
# failing every time it reaches the multiply.
%extern.C proc exit [let $code: i32];
fn double [let $p: i64] -> i64:
    ret $p x 2;
;fn double
MAIN [let $argc: i32, let $argv: ptr[ptr[u8]]]:
    var $code: i64 = double(1);
    if $argc > 5:
        change $code to double(5000000000000000000);
    ;if
    no_check my.exit(to.i32($code mod.floor 256));
;MAIN
