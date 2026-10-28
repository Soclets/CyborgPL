fn g [let $p: u8] -> u8:
    if false:
        let shadow $p: bool := true;
        ret $p;
    ;if
    ret $p;
;fn g
MAIN:
    let $x := g(1);
;MAIN
