# A held function that can fail (functions-as-values.md: the failure list is
# part of the type): called as $f(…) with otherwise, named or a literal, a
# reason carrying its payload out.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
fn half [let $n: i32] -> i32 or fail[odd: i32]:
    if ($n mod.floor 2) == 1:
        fail odd[$n];
    ;if
    ret $n div.floor 2;
;fn half
fn third [let $n: i32] -> i32 or fail[odd: i32]:
    if ($n mod.floor 3) != 0:
        fail odd[$n];
    ;if
    ret $n div.floor 3;
;fn third
fn apply [let $f: fn[i32] -> i32 or fail[odd: i32], let $n: i32] -> i32:
    let $v := $f($n)-
    otherwise:
        case odd $x:
            ret 0 - $x;
        ;case odd
    ;otherwise;
    ret $v;
;fn apply
MAIN:
    let $k := "SYNAPSE_SURELY_UNSET_VARIABLE";
    no_check let $there := ptr.is_null(getenv(lend $k));
    var $n: i32 = 7;
    if $there:
        change $n to 12;
    ;if
    var $got: i32 = 0;
    if apply(half, $n) == 6 and apply(half, $n + 1) == -13:
        change $got to $got + 1;
    ;if
    if apply(third, $n) == 4 and apply(third, $n + 1) == -13:
        change $got to $got + 2;
    ;if
    let $g := fn [let $m: i32] -> i32 or fail[odd: i32]:
        if $m > 10:
            fail odd[$m];
        ;if
        ret $m;
    ;fn;
    if apply($g, 3) == 3 and apply($g, $n) == -12:
        change $got to $got + 4;
    ;if
    let $direct := $g($n) ?? 99;
    if $direct == 99:
        change $got to $got + 8;
    ;if
    no_check my.exit($got);
;MAIN
