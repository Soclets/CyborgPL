# A character literal of several code points can only be a grapheme
# (types.md), with its unit unwritten too: e + a combining accent.
%extern.C proc exit [let $code: i32];
MAIN:
    let $g := `é`;
    let $t := to.str.utf8(lend $g);
    var $got: i32 = 0;
    if std::str.bytes(lend $t) == 3 and std::str.clusters(lend $t) == 1:
        change $got to $got + 1;
    ;if
    if $g == `é`:
        change $got to $got + 2;
    ;if
    no_check my.exit($got);
;MAIN
