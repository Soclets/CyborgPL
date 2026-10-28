%extern.C proc f [let $a: i32] -> i32;
MAIN:
    no_check let $x := f(1) ?? 0;
;MAIN
