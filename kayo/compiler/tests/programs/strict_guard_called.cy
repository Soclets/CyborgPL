# A strict function's own guard stands even when every call settles it
# (check claude, seeds 2875 and 2937): called only as f[5], `'p' !== 0` was
# folded to true for the whole program, and layer 3 then asked about
# `16 mod 'p'` with no `if` and refused it for p = 0. f[5]: 16 mod 5 is 1,
# not >= 3, so false; g[5] and h[5] the same; 1 + 2 + 4 when each is false.
%extern.C proc exit [let $code: i32];
fn f [let $p: i32] -> bool:
    if $p != 0:
        ret (16 mod.floor $p) >= 3;
    ;if
    ret false;
;fn f
fn g [let $p: i32] -> bool:
    ret $p != 0 and.then ((16 mod.floor $p) >= 3);
;fn g
fn h [let $p: i32] -> bool:
    ret $p == 0 or.else ((16 mod.floor $p) >= 3);
;fn h
MAIN:
    var $code: i32 = 0;
    if not f(5):
        change $code to $code + 1;
    ;if
    if not g(5):
        change $code to $code + 2;
    ;if
    if not h(5):
        change $code to $code + 4;
    ;if
    no_check my.exit($code);
;MAIN
