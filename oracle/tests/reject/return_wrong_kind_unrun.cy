fn g [let $p: u8] -> u8:
    if false:
        ret true;
    ;if
    ret 1;
;fn g
MAIN:
    let $x := g(1);
;MAIN
