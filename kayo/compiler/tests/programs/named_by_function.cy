# A template's name worked out by a function of the file, run while
# generating (owner, 2026-10-13: `named size_name($n);`): kb, mb and pow6.
# 1 + 1 + 64 is 66.
%extern.C proc exit [let $code: i32];

fn size_name [let $n: i32] -> str.utf8:
    if $n == 1024:
        ret "kb";
    ;if
    if $n == 1048576:
        ret "mb";
    ;if
    var $k = 0;
    var $m = $n;
    while $m > 1:
        change $m to $m div.floor 2;
        change $k to $k + 1;
    ;while
    ret "pow" concat to.stored.str.utf8($k);
;fn size_name

coll.fn block:
    for $n in [1024, 1048576, 64]:
        named size_name($n);
        member [] -> i32:
            ret $n;
        ;member
    ;for
    #generated_9_lines_below
    member block.kb [] -> i32:
        ret 1024;
    ;member block.kb
    member block.mb [] -> i32:
        ret 1048576;
    ;member block.mb
    member block.pow6 [] -> i32:
        ret 64;
    ;member block.pow6
;coll.fn block

MAIN:
    let $t := block.kb() div.floor 1024 + block.mb() div.floor 1048576 + block.pow6();
    no_check my.exit(to.i32($t));
;MAIN
