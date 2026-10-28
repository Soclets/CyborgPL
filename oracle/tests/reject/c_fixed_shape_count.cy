%extern.C proc p_one = %C.printf [let $format: ptr[u8], let ...$n: i32] -> i32;

MAIN:
    let $f := %C.str.make("%d %d");
    no_check p_one(lend $f, 1, 2);
;MAIN
