# Insensitive means Unicode's full case folding (owner, 2026-10-06): `ß`
# matches `SS`, `É` matches `é`. A lone file has WSTN. 7 + 7 = 14.
case.any [fn, var];
%extern.C proc exit [let $code: i32];

fn Größe [] -> i32:
    ret 7;
;fn Größe

MAIN:
    let $Café := GRÖSSE();
    no_check my.exit($CAFÉ + größe());
;MAIN
