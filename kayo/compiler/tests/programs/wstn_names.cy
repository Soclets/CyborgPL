# Names in every script, and emoji (owner, 2026-10-06: WSTN, "`b`"); a lone
# file has every core. A Thai function, a Devanagari name with its combining
# marks, an emoji: 3 + 4 x 10 = 43.
%extern.C proc exit [let $code: i32];
fn บวก [let $ก: i32, let $ข: i32] -> i32:
    ret $ก + $ข;
;fn บวก
MAIN:
    let $'🍕': i32 := 3;
    let $'नमस्ते' := บวก(1, 3);
    no_check my.exit($'🍕' + $'नमस्ते' x 10);
;MAIN
